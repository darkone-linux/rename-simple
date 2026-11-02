//! Windows-specific behaviour.
//!
//! NTFS is case-insensitive: `Photo.JPG` and `photo.jpg` are two spellings of
//! one file, so a case-only rename meets a destination that "already exists"
//! and is the source itself.
#![cfg(windows)]

use assert_cmd::Command;
use rename_files::{compare_entries, EntryMatch};
use std::fs;

fn cmd() -> Command {
    Command::cargo_bin("rename-simple").unwrap()
}

#[test]
fn test_two_spellings_of_one_file_are_the_same_entry() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Photo.JPG"), "the only copy").unwrap();

    let found = compare_entries(&dir.join("Photo.JPG"), &dir.join("photo.jpg")).unwrap();

    assert_eq!(found, EntryMatch::SameEntry);
}

#[test]
fn test_delete_duplicates_keeps_the_only_copy_on_a_case_only_rename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Photo.JPG"), "the only copy").unwrap();

    let output = cmd().arg("-D").arg(dir.join("Photo.JPG")).output().unwrap();

    assert!(output.status.success());
    let names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1, "the file must survive, found {names:?}");
    assert_eq!(
        fs::read_to_string(dir.join("Photo.JPG")).unwrap(),
        "the only copy"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("duplicate removed"),
        "nothing may be removed: {stdout}"
    );
}

#[test]
fn test_distinct_identical_files_are_still_duplicates() {
    // The identity check must not hide real duplicates: two distinct files
    // with the same bytes are still `Identical`.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("IMG 1.mov"), "same bytes").unwrap();
    fs::write(dir.join("img-1.mov"), "same bytes").unwrap();

    let found = compare_entries(&dir.join("IMG 1.mov"), &dir.join("img-1.mov")).unwrap();

    assert_eq!(found, EntryMatch::Identical);
}
