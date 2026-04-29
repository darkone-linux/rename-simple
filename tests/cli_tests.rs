use assert_cmd::Command;
use std::fs;

fn cmd() -> Command {
    Command::cargo_bin("rename-simple").unwrap()
}

#[test]
fn test_files_renamed_by_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();
    fs::write(dir.join("Café.md"), "content").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("fichier-test.txt").exists());
    assert!(dir.join("cafe.md").exists());
}

#[test]
fn test_dirs_renamed_by_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Répertoire Test")).unwrap();
    fs::create_dir(dir.join("Café")).unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("repertoire-test").exists());
    assert!(dir.join("cafe").exists());
}

#[test]
fn test_files_only_with_f_flag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier.txt"), "content").unwrap();
    fs::create_dir(dir.join("Répertoire")).unwrap();

    let output = cmd().arg("-f").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("fichier.txt").exists());
    assert!(!dir.join("repertoire").exists());
}

#[test]
fn test_dirs_only_with_d_flag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier.txt"), "content").unwrap();
    fs::create_dir(dir.join("Répertoire")).unwrap();

    let output = cmd().arg("-d").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(!dir.join("fichier.txt").exists());
    assert!(dir.join("repertoire").exists());
}

#[test]
fn test_dry_run_no_actual_rename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier.txt"), "content").unwrap();
    let original_content = fs::read(dir.join("Fichier.txt")).unwrap();

    let output = cmd().arg("-n").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("Fichier.txt").exists());
    assert!(!dir.join("fichier.txt").exists());
    assert_eq!(fs::read(dir.join("Fichier.txt")).unwrap(), original_content);
}

#[test]
fn test_hidden_files_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join(".hidden"), "content").unwrap();
    fs::write(dir.join("visible.txt"), "content").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join(".hidden").exists());
    assert!(dir.join("visible.txt").exists());
}

#[test]
fn test_conflict_warning() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "content1").unwrap();
    fs::write(dir.join("CAFÉ.TXT"), "content2").unwrap();

    let output = cmd().arg("-n").arg(dir).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.to_uppercase().contains("CONFLICT"));
}

#[test]
fn test_existing_destination_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "content1").unwrap();
    fs::write(dir.join("cafe.txt"), "content2").unwrap();

    let output = cmd().arg("-n").arg(dir).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_uppercase().contains("CONFLICT") || combined.to_uppercase().contains("EXISTS")
    );
}

#[test]
fn test_help_flag() {
    let output = cmd().arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("Usage:"));
    assert!(combined.contains("--dry-run"));
}

#[test]
fn test_no_extension_file_renamed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Mon Fichier"), "content").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier").exists());
}

#[test]
fn test_compound_extension_preserved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("archive.tar.gz"), "content").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("archive.tar.gz").exists());
}

#[test]
fn test_numbers_preserved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("File 2024.txt"), "content").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("file-2024.txt").exists());
}
