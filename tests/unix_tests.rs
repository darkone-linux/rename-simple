#![cfg(unix)]

use assert_cmd::Command;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;

fn cmd() -> Command {
    Command::cargo_bin("rename-simple").unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// Symlink handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_symlink_to_file_renames_link_not_target() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("target.txt"), "TARGET CONTENT").unwrap();
    std::os::unix::fs::symlink("target.txt", dir.join("Lien Étrange.txt")).unwrap();

    let output = cmd().arg(dir.join("Lien Étrange.txt")).output().unwrap();

    assert!(output.status.success());

    // The original symlink name is gone
    assert!(!dir.join("Lien Étrange.txt").exists());

    // The renamed symlink exists and still resolves to the original file
    let renamed = dir.join("lien-etrange.txt");
    assert!(renamed.exists(), "renamed symlink should exist");
    assert_eq!(
        fs::read_to_string(&renamed).unwrap(),
        "TARGET CONTENT",
        "symlink must still resolve to the untouched target"
    );

    // The symlink is still a symlink (not a copy of the target)
    let meta = fs::symlink_metadata(&renamed).unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "renamed entry must remain a symlink"
    );

    // The target file itself is untouched
    assert!(dir.join("target.txt").exists());
    assert_eq!(
        fs::read_to_string(dir.join("target.txt")).unwrap(),
        "TARGET CONTENT"
    );
}

#[test]
fn test_dangling_symlink_is_left_alone() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    // Symlink to a non-existent target. is_file()/is_dir() (and exists())
    // return false for dangling symlinks, so it is never renamed.
    std::os::unix::fs::symlink("does-not-exist", dir.join("Dangling Link.txt")).unwrap();
    fs::write(dir.join("Real File.txt"), "x").unwrap();

    let output = cmd()
        .arg(dir.join("Dangling Link.txt"))
        .arg(dir.join("Real File.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("real-file.txt").exists());
    // Dangling symlink stays put under its original name
    assert!(fs::symlink_metadata(dir.join("Dangling Link.txt")).is_ok());
}

#[test]
fn test_broken_symlink_argument_is_reported_as_such() {
    // The link exists, only its target does not: "No such file" would send
    // the user looking for a typo.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    std::os::unix::fs::symlink("does-not-exist", dir.join("Dangling Link.txt")).unwrap();

    let output = cmd().arg(dir.join("Dangling Link.txt")).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]") && stderr.contains("Broken symbolic link"),
        "expected a broken-link error, got: {stderr}"
    );
    assert!(!stderr.contains("No such file"), "got: {stderr}");
}

#[test]
fn test_dangling_symlink_at_the_destination_takes_the_name() {
    // `my-file.txt` points to nothing, but the name is taken all the same: the
    // dry-run must not announce a rename, and the real run must report the
    // clash rather than a raw "File exists" from the kernel.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("My File.txt"), "content").unwrap();
    std::os::unix::fs::symlink("does-not-exist", dir.join("my-file.txt")).unwrap();

    for args in [&["-n"][..], &[]] {
        let output = cmd()
            .args(args)
            .arg(dir.join("My File.txt"))
            .output()
            .unwrap();

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stdout.contains("[R]"), "{args:?}: no rename: {stdout}");
        assert!(
            stderr.contains("[E]") && stderr.contains("File name already exists"),
            "{args:?}: expected the name clash, got: {stderr}"
        );
    }
    assert!(dir.join("My File.txt").exists());
    assert!(fs::symlink_metadata(dir.join("my-file.txt"))
        .unwrap()
        .file_type()
        .is_symlink());
}

// ─────────────────────────────────────────────────────────────────────────────
// Failure modes — read-only parent (audit M3 surface)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_readonly_parent_yields_error_without_panic() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let file = dir.join("Mon Fichier.txt");
    fs::write(&file, "x").unwrap();

    // Make the parent read-only: rename of the entry inside it must fail
    // (EACCES) but the program must not panic.
    let mut perms = fs::metadata(dir).unwrap().permissions();
    perms.set_mode(0o555);
    fs::set_permissions(dir, perms).unwrap();

    // If perms can be bypassed (e.g. running as root, special FS), skip.
    let probe_ok = fs::write(dir.join(".probe"), b"x").is_ok();
    if probe_ok {
        let _ = fs::remove_file(dir.join(".probe"));
        let mut restore = fs::metadata(dir).unwrap().permissions();
        restore.set_mode(0o755);
        fs::set_permissions(dir, restore).unwrap();
        eprintln!("skipping: chmod 0555 did not take effect on this environment");
        return;
    }

    let output = cmd().arg(&file).output().unwrap();

    // Restore writable perms BEFORE the temp dir tries to drop itself,
    // otherwise tempfile cannot clean up.
    let mut restore = fs::metadata(dir).unwrap().permissions();
    restore.set_mode(0o755);
    fs::set_permissions(dir, restore).unwrap();

    assert!(
        output.status.success(),
        "exit must stay 0 on per-file errors"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("error"),
        "per-file failure must be reported on stderr, got: {stderr}"
    );
}

#[test]
fn test_unreadable_parent_reports_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let locked = temp_dir.path().join("Locked");
    fs::create_dir(&locked).unwrap();
    fs::write(locked.join("File A.txt"), "x").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

    // Root reads through mode 000: nothing to test then.
    let bypassed = fs::symlink_metadata(locked.join("File A.txt")).is_ok();
    let output = (!bypassed).then(|| cmd().arg(locked.join("File A.txt")).output().unwrap());

    // Restore the mode before anything can panic, or tempfile cannot clean up.
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    let Some(output) = output else {
        eprintln!("skipping: mode 000 does not stop this user");
        return;
    };

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]") && stderr.contains("Permission denied"),
        "expected a permission error, got: {stderr}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Invalid UTF-8 filenames
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_invalid_utf8_filename_does_not_panic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    // 0xFF / 0xFE are invalid as part of any UTF-8 byte sequence.
    let bad_bytes: &[u8] = &[b'b', b'a', b'd', 0xff, 0xfe, b'.', b't', b'x', b't'];
    let bad_name = OsStr::from_bytes(bad_bytes);
    let bad_path = dir.join(bad_name);
    fs::write(&bad_path, "x").unwrap();

    // Add a normal file alongside so we can confirm it still gets renamed.
    fs::write(dir.join("Bon Fichier.txt"), "y").unwrap();

    let output = cmd()
        .arg(&bad_path)
        .arg(dir.join("Bon Fichier.txt"))
        .output()
        .unwrap();

    assert!(output.status.success(), "must not crash on invalid UTF-8");
    assert!(
        bad_path.exists(),
        "invalid-UTF-8 filename must be skipped, not renamed or deleted"
    );
    assert!(
        dir.join("bon-fichier.txt").exists(),
        "valid neighbours must still be renamed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Duplicate handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_hard_link_to_destination_is_not_deleted() {
    // Source and destination are two names for the *same* inode: deleting the
    // source would be a rename in disguise, so it must stay a plain error.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("cafe.txt"), "shared").unwrap();
    fs::hard_link(dir.join("cafe.txt"), dir.join("Café.txt")).unwrap();

    let output = cmd().arg("-D").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("Café.txt").exists(), "hard link must stay");
    assert!(dir.join("cafe.txt").exists(), "destination must stay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
    assert!(!stderr.contains("[W]"));
}

#[test]
fn test_hard_link_spelled_in_another_case_is_not_renamed() {
    // On a case-sensitive filesystem `REPORT.txt` and `report.txt` are two
    // entries, here two hard links of one file. Renaming would drop one of
    // the two names: still a conflict, even though both resolve to the same
    // entry as on a case-insensitive filesystem.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("report.txt"), "shared").unwrap();
    fs::hard_link(dir.join("report.txt"), dir.join("REPORT.txt")).unwrap();

    let output = cmd().arg(dir.join("REPORT.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("REPORT.txt").exists(), "hard link must stay");
    assert!(dir.join("report.txt").exists(), "destination must stay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
}

#[test]
fn test_symlink_duplicate_of_its_own_target_is_not_deleted() {
    // A symlink whose cleaned name points at its own target resolves to the
    // same inode: same-entry detection must win over content comparison.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("cafe.txt"), "shared").unwrap();
    std::os::unix::fs::symlink("cafe.txt", dir.join("Café.txt")).unwrap();

    let output = cmd().arg("-D").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("Café.txt").exists(), "symlink must stay");
    assert_eq!(fs::read_to_string(dir.join("cafe.txt")).unwrap(), "shared");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
}

#[test]
fn test_symlink_source_is_never_deleted_as_duplicate() {
    // The source is a symlink to a third file whose content happens to match
    // the destination. A symlink is not a redundant copy of anything: deleting
    // it destroys a link the user created, and the rename it stands in for
    // would have moved the link, not duplicated the bytes.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("real.txt"), "CONTENT").unwrap();
    std::os::unix::fs::symlink("real.txt", dir.join("Café.txt")).unwrap();
    fs::write(dir.join("cafe.txt"), "CONTENT").unwrap();

    let output = cmd().arg("-D").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(
        dir.join("Café.txt").symlink_metadata().is_ok(),
        "the symlink must survive"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
    assert!(!stderr.contains("[W]"), "nothing may be deleted here");
}

#[test]
fn test_symlinked_destination_never_deletes_the_source() {
    // The destination name is a symlink pointing outside the operation. Its
    // content matches, but deleting the source would leave the last *real*
    // copy of those bytes outside the directory being cleaned up — one
    // `rm elsewhere.txt` away from being lost for good.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("elsewhere.txt"), "CONTENT").unwrap();
    std::os::unix::fs::symlink("elsewhere.txt", dir.join("cafe.txt")).unwrap();
    fs::write(dir.join("Café.txt"), "CONTENT").unwrap();

    let output = cmd().arg("-D").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(
        dir.join("Café.txt").exists(),
        "the only real copy must survive"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
    assert!(!stderr.contains("[W]"), "nothing may be deleted here");
}

/// Two files with the same bytes: `Secret Notes.txt`, private (0600), and its
/// cleaned name `secret-notes.txt`, writable by anyone (0666).
fn private_source_and_open_duplicate() -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Secret Notes.txt"), "same bytes").unwrap();
    fs::write(dir.join("secret-notes.txt"), "same bytes").unwrap();
    fs::set_permissions(
        dir.join("Secret Notes.txt"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::set_permissions(
        dir.join("secret-notes.txt"),
        fs::Permissions::from_mode(0o666),
    )
    .unwrap();
    temp_dir
}

#[test]
fn test_duplicate_with_other_permissions_is_not_deleted() {
    // Deleting the private source would leave only a copy every user can
    // change, possibly planted beforehand in a shared directory: -D keeps
    // both, and the error says why.
    let temp_dir = private_source_and_open_duplicate();
    let dir = temp_dir.path();

    for flags in ["-nD", "-D"] {
        let output = cmd()
            .arg(flags)
            .arg(dir.join("Secret Notes.txt"))
            .output()
            .unwrap();

        assert!(output.status.success());
        assert!(
            dir.join("Secret Notes.txt").exists(),
            "{flags}: the private source must stay"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("[E]") && stderr.contains("permissions"),
            "{flags}: expected an [E] line about permissions, got: {stderr}"
        );
        assert!(
            !stderr.contains("[W]"),
            "{flags}: nothing may be deleted: {stderr}"
        );
    }
}

#[test]
fn test_duplicate_with_other_permissions_does_not_suggest_d() {
    // -D would not delete this source either: the error must not point at it.
    let temp_dir = private_source_and_open_duplicate();
    let dir = temp_dir.path();

    let output = cmd().arg(dir.join("Secret Notes.txt")).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]") && !stderr.contains("-D"),
        "expected an [E] line without the -D hint, got: {stderr}"
    );
}

#[test]
fn test_duplicate_with_other_group_is_not_deleted() {
    // Same permission bits, another group: the kept copy would hand the
    // group's access to other people.
    use std::os::unix::fs::MetadataExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Group Notes.txt"), "same bytes").unwrap();
    fs::write(dir.join("group-notes.txt"), "same bytes").unwrap();

    // Any other group the user belongs to will do; without one, or where the
    // filesystem refuses the change, there is nothing to test.
    let current = fs::metadata(dir.join("group-notes.txt")).unwrap().gid();
    let other = std::process::Command::new("id")
        .arg("-G")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|groups| {
            groups
                .split_whitespace()
                .filter_map(|g| g.parse::<u32>().ok())
                .find(|&g| g != current)
        });
    let Some(other) = other else {
        eprintln!("skipping: the user belongs to a single group");
        return;
    };
    if std::os::unix::fs::chown(dir.join("group-notes.txt"), None, Some(other)).is_err() {
        eprintln!("skipping: cannot change the group on this filesystem");
        return;
    }

    let output = cmd()
        .arg("-D")
        .arg(dir.join("Group Notes.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("Group Notes.txt").exists(), "the source must stay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]") && !stderr.contains("[W]"),
        "expected an [E] line and no deletion, got: {stderr}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Reporting of names that cannot be processed
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_invalid_utf8_name_is_reported() {
    // An entry named explicitly on the command line must never vanish without
    // a word: it cannot be renamed, so it is reported as an error.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    let bad_path = dir.join(OsStr::from_bytes(&[
        b'b', b'a', b'd', 0xff, b'.', b't', b'x', b't',
    ]));
    fs::write(&bad_path, "x").unwrap();

    let output = cmd().arg(&bad_path).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]") && stderr.contains("UTF-8"),
        "expected an [E] line about UTF-8, got: {stderr}"
    );
    assert!(bad_path.exists(), "the entry itself must be left alone");
}

#[test]
fn test_type_filter_stays_silent_on_invalid_utf8() {
    // Filtered out by -d: the invalid name is irrelevant, so it must not be
    // reported as an error.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    let bad_path = dir.join(OsStr::from_bytes(&[
        b'b', b'a', b'd', 0xff, b'.', b't', b'x', b't',
    ]));
    fs::write(&bad_path, "x").unwrap();

    let output = cmd().arg("-d").arg(&bad_path).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "expected no output, got: {stderr}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Names printed on the terminal
// ─────────────────────────────────────────────────────────────────────────────

/// Fail if `stream` carries a character a file name must never send raw to the
/// terminal: C0 and C1 controls (ESC, BEL, CSI…), line and paragraph
/// separators, bidirectional formatting characters. The newlines ending the
/// report lines are the only controls allowed.
fn assert_no_raw_controls(stream: &str) {
    let raw = stream.chars().find(|&c| {
        (c.is_control() && c != '\n')
            || matches!(
                c,
                '\u{061c}'
                    | '\u{200e}'
                    | '\u{200f}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{202a}'..='\u{202e}'
                    | '\u{2066}'..='\u{2069}'
            )
    });
    assert_eq!(raw, None, "raw control character in: {stream:?}");
}

#[test]
fn test_escape_sequences_in_a_name_are_printed_escaped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    // Clears the screen, then sets the terminal title.
    let name = "Evil\x1b[2J\x1b]0;pwned\x07 name.txt";
    fs::write(dir.join(name), "x").unwrap();

    let output = cmd().arg("-n").arg(dir.join(name)).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_no_raw_controls(&stdout);
    assert!(
        stdout.contains(r"Evil\u{1b}[2J\u{1b}]0;pwned\u{7} name.txt -> "),
        "expected the escaped name, got: {stdout:?}"
    );
}

#[test]
fn test_newline_in_a_name_cannot_forge_a_report_line() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let name = "Two\n[R] .: fake.txt -> fake.txt";
    fs::write(dir.join(name), "x").unwrap();

    let output = cmd().arg("-n").arg(dir.join(name)).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "one [R] line, then the summary: {stdout:?}");
    assert!(
        lines[0].contains(r"Two\n[R] .: fake.txt -> fake.txt -> "),
        "expected the escaped name, got: {stdout:?}"
    );
}

#[test]
fn test_control_characters_in_the_directory_are_printed_escaped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let parent = temp_dir.path().join("Evil\x1b]0;pwned\x07");
    fs::create_dir(&parent).unwrap();
    fs::write(parent.join("File A.txt"), "x").unwrap();

    let output = cmd()
        .arg("-n")
        .arg(parent.join("File A.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_no_raw_controls(&stdout);
    assert!(
        stdout.contains(r"Evil\u{1b}]0;pwned\u{7}: File A.txt -> file-a.txt"),
        "expected the escaped directory, got: {stdout:?}"
    );
}

#[test]
fn test_control_characters_in_an_error_line_are_printed_escaped() {
    let temp_dir = tempfile::tempdir().unwrap();
    // Not valid UTF-8 either, so the entry is reported as an error on stderr.
    let bad_path = temp_dir
        .path()
        .join(OsStr::from_bytes(b"Bad\x1b[2J\xff.txt"));
    fs::write(&bad_path, "x").unwrap();

    let output = cmd().arg(&bad_path).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_no_raw_controls(&stderr);
    assert!(
        stderr.contains(r"Bad\u{1b}[2J"),
        "expected the escaped name, got: {stderr:?}"
    );
}

#[test]
fn test_bidi_and_c1_controls_in_a_name_are_printed_escaped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    // RIGHT-TO-LEFT OVERRIDE displays the rest of the line reversed, U+009B
    // is the 8-bit CSI some terminals honour, U+2028 breaks the line in some
    // viewers.
    let name = "Report\u{202e}fdp\u{9b}2J\u{2028}x.exe";
    fs::write(dir.join(name), "x").unwrap();

    let output = cmd().arg("-n").arg(dir.join(name)).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_no_raw_controls(&stdout);
    assert!(
        stdout.contains(r"Report\u{202e}fdp\u{9b}2J\u{2028}x.exe -> "),
        "expected the escaped name, got: {stdout:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Output that cannot be written (`| head` exited)
// ─────────────────────────────────────────────────────────────────────────────

/// A stream whose reader is gone: one end of a socket pair whose other end is
/// already closed. Every write fails with EPIPE, as with a pipe whose reader
/// (`| head -1`) has exited, but from the very first byte.
fn closed_pipe() -> std::process::Stdio {
    let (writer, reader) = std::os::unix::net::UnixStream::pair().unwrap();
    drop(reader);
    std::process::Stdio::from(std::os::fd::OwnedFd::from(writer))
}

/// Twenty files to rename: a run stopped by its first failed write would
/// leave most of them behind.
fn twenty_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    (1..=20)
        .map(|i| {
            let path = dir.join(format!("File {i:02}.txt"));
            fs::write(&path, "x").unwrap();
            path
        })
        .collect()
}

fn assert_all_renamed(dir: &std::path::Path) {
    for i in 1..=20 {
        assert!(
            dir.join(format!("file-{i:02}.txt")).exists(),
            "File {i:02}.txt was not renamed"
        );
    }
}

#[test]
fn test_closed_stdout_does_not_stop_the_renames() {
    let temp_dir = tempfile::tempdir().unwrap();
    let files = twenty_files(temp_dir.path());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rename-simple"))
        .args(&files)
        .stdout(closed_pipe())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{:?}: {stderr}", output.status);
    assert!(stderr.is_empty(), "nothing to report on stderr: {stderr}");
    assert_all_renamed(temp_dir.path());
}

#[test]
fn test_closed_stderr_does_not_stop_the_renames() {
    let temp_dir = tempfile::tempdir().unwrap();
    let files = twenty_files(temp_dir.path());

    // The missing entry is reported on stderr before anything is renamed.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rename-simple"))
        .arg(temp_dir.path().join("Missing File.txt"))
        .args(&files)
        .stderr(closed_pipe())
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output.status);
    assert_all_renamed(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("21 entries matched, 20 entries renamed, 1 error."),
        "stdout must still carry the report, got: {stdout}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Filesystems without RENAME_NOREPLACE (NFS, sshfs, ntfs-3g)
// ─────────────────────────────────────────────────────────────────────────────

/// Run rename-simple under `strace`, every `renameat2` call failing with
/// `errno`: what NFS or a FUSE filesystem without `rename2` (sshfs, ntfs-3g)
/// answers to `RENAME_NOREPLACE`, or a kernel without `renameat2` (ENOSYS).
/// `None` when strace is missing or cannot trace here.
#[cfg(target_os = "linux")]
fn run_with_renameat2_failing(
    errno: &str,
    args: &[&std::path::Path],
) -> Option<std::process::Output> {
    let bin = env!("CARGO_BIN_EXE_rename-simple");
    let strace = |extra: &[&str]| {
        let mut command = std::process::Command::new("strace");
        command
            .args(["-f", "-qq", "-o", "/dev/null"])
            .args(extra)
            .arg(bin);
        command
    };
    let probe = strace(&[]).arg("--version").output().ok()?;
    if !probe.status.success() {
        return None;
    }
    let inject = format!("inject=renameat2:error={errno}");
    Some(
        strace(&["-e", "trace=renameat2", "-e", &inject])
            .args(args)
            .output()
            .unwrap(),
    )
}

#[cfg(target_os = "linux")]
#[test]
fn test_rename_falls_back_when_noreplace_is_unsupported() {
    for errno in ["EINVAL", "ENOSYS", "EOPNOTSUPP"] {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();
        fs::write(dir.join("My File.txt"), "content").unwrap();
        fs::create_dir(dir.join("My Dir")).unwrap();

        let Some(output) =
            run_with_renameat2_failing(errno, &[&dir.join("My File.txt"), &dir.join("My Dir")])
        else {
            eprintln!("skipping: strace is missing or cannot trace here");
            return;
        };

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{errno}: {stderr}");
        assert!(
            stderr.is_empty(),
            "{errno}: no error expected, got: {stderr}"
        );
        assert_eq!(
            fs::read_to_string(dir.join("my-file.txt")).unwrap(),
            "content",
            "{errno}"
        );
        assert!(dir.join("my-dir").is_dir(), "{errno}");
    }
}
