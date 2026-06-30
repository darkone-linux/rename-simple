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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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

    // Symlink to a non-existent target. is_file()/is_dir() return false for
    // dangling symlinks, so compute_renames should silently skip it.
    std::os::unix::fs::symlink("does-not-exist", dir.join("Dangling Link.txt")).unwrap();
    fs::write(dir.join("Real File.txt"), "x").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("real-file.txt").exists());
    // Dangling symlink stays put under its original name
    assert!(fs::symlink_metadata(dir.join("Dangling Link.txt")).is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// Symlinks — recursion must not escape the target tree (audit C2)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_recursive_does_not_descend_into_subdir_symlink() {
    // Layout:
    //   <root>/
    //     inside/
    //       link -> <root>/outside
    //     outside/
    //       Sécret Fichier.txt
    //
    // After `rename-simple -a -r <root>/inside`, the file inside `outside/`
    // MUST NOT be renamed — recursion must never follow a symlinked dir.
    let root = tempfile::tempdir().unwrap();
    let inside = root.path().join("inside");
    let outside = root.path().join("outside");
    fs::create_dir(&inside).unwrap();
    fs::create_dir(&outside).unwrap();

    fs::write(outside.join("Sécret Fichier.txt"), "secret").unwrap();
    std::os::unix::fs::symlink(&outside, inside.join("link")).unwrap();

    let output = cmd().arg("-a").arg("-r").arg(&inside).output().unwrap();

    assert!(output.status.success());
    assert!(
        outside.join("Sécret Fichier.txt").exists(),
        "recursion through a directory symlink must not rename files outside the target tree"
    );
    assert!(
        !outside.join("secret-fichier.txt").exists(),
        "no renamed twin must appear in the external directory"
    );
}

#[test]
fn test_recursive_symlink_loop_terminates() {
    // dir/a -> dir/b, dir/b -> dir/a — a classic cycle. The program must
    // terminate (Linux returns ELOOP after 40 hops; we still rely on the
    // C2 fix to avoid following directory symlinks in the first place).
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    std::os::unix::fs::symlink(dir.join("b"), dir.join("a")).unwrap();
    std::os::unix::fs::symlink(dir.join("a"), dir.join("b")).unwrap();

    fs::write(dir.join("Real File.txt"), "x").unwrap();

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

    assert!(output.status.success(), "must terminate without panic");
    assert!(dir.join("real-file.txt").exists());
}

// ─────────────────────────────────────────────────────────────────────────────
// Failure modes — read-only parent (audit M3 surface)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_readonly_parent_yields_error_without_panic() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();

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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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

// ─────────────────────────────────────────────────────────────────────────────
// Invalid UTF-8 (audit M2 — verbose warning)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_invalid_utf8_filename_reports_warning_in_verbose() {
    // Same setup as test_invalid_utf8_filename_does_not_panic, but with -v:
    // the user must be told *which* entry was skipped instead of the rename
    // silently dropping it.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let bad_bytes: &[u8] = &[b'b', b'a', b'd', 0xff, 0xfe, b'.', b't', b'x', b't'];
    let bad_name = OsStr::from_bytes(bad_bytes);
    fs::write(dir.join(bad_name), "x").unwrap();
    fs::write(dir.join("Bon Fichier.txt"), "y").unwrap();

    let output = cmd().arg("-a").arg("-v").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("skip")
            || stderr.to_lowercase().contains("non-utf"),
        "verbose mode must surface the skipped non-UTF-8 entry, stderr was: {stderr}"
    );
}

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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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
