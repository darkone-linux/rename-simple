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

    let output = cmd().arg("-a").arg(dir).output().unwrap();

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

    let output = cmd().arg("-a").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("real-file.txt").exists());
    // Dangling symlink stays put under its original name
    assert!(fs::symlink_metadata(dir.join("Dangling Link.txt")).is_ok());
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

    let output = cmd().arg("-a").arg(dir).output().unwrap();

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
