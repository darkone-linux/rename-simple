use assert_cmd::Command;
use std::fs;

fn cmd() -> Command {
    Command::cargo_bin("rename-simple").unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// Recursive processing (-r flag)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_recursive_processes_nested_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Sous-dossier")).unwrap();
    fs::write(dir.join("Sous-dossier/Fichier Test.txt"), "content").unwrap();

    let output = cmd().arg("-r").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("sous-dossier/fichier-test.txt").exists());
}

#[test]
fn test_recursive_processes_deeply_nested_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir_all(dir.join("niveau1/niveau2/niveau3")).unwrap();
    fs::write(
        dir.join("niveau1/niveau2/niveau3/Document Test.txt"),
        "content",
    )
    .unwrap();

    let output = cmd().arg("-r").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir
        .join("niveau1/niveau2/niveau3/document-test.txt")
        .exists());
}

#[test]
fn test_recursive_renames_nested_directories() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir_all(dir.join("Sous-dossier/Sous-sous-dossier")).unwrap();
    fs::write(dir.join("test.txt"), "content").unwrap();

    let output = cmd().arg("-r").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("sous-dossier/sous-sous-dossier").exists());
}

#[test]
fn test_recursive_with_dry_run_no_actual_rename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Sous-dossier")).unwrap();
    fs::write(dir.join("Sous-dossier/Fichier.txt"), "content").unwrap();
    let original_content = fs::read(dir.join("Sous-dossier/Fichier.txt")).unwrap();

    let output = cmd().arg("-r").arg("-n").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("Sous-dossier/Fichier.txt").exists());
    assert!(!dir.join("sous-dossier/fichier.txt").exists());
    assert_eq!(
        fs::read(dir.join("Sous-dossier/Fichier.txt")).unwrap(),
        original_content
    );
}

#[test]
fn test_recursive_files_only_with_f_and_r() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Répertoire")).unwrap();
    fs::write(dir.join("Répertoire/Fichier.txt"), "content").unwrap();
    fs::write(dir.join("Fichier.txt"), "content").unwrap();

    let output = cmd().arg("-f").arg("-r").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("fichier.txt").exists());
    assert!(dir.join("Répertoire").exists());
}

#[test]
fn test_recursive_dirs_only_with_d_and_r() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Répertoire")).unwrap();
    fs::create_dir(dir.join("Sous-dossier")).unwrap();
    fs::write(dir.join("Fichier.txt"), "content").unwrap();

    let output = cmd().arg("-d").arg("-r").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("repertoire").exists());
    assert!(dir.join("sous-dossier").exists());
    assert!(dir.join("Fichier.txt").exists());
}

#[test]
fn test_non_recursive_renames_root_only() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Sous-dossier")).unwrap();
    fs::write(dir.join("Sous-dossier/Fichier Test.txt"), "content").unwrap();
    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();

    let output = cmd().arg("-n").arg(dir).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fichier-test.txt"));
    assert!(stdout.contains("sous-dossier"));
}

#[test]
fn test_recursive_handles_multiple_subdirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Dossier A")).unwrap();
    fs::create_dir(dir.join("Dossier B")).unwrap();
    fs::write(dir.join("Dossier A/Fichier A.txt"), "content").unwrap();
    fs::write(dir.join("Dossier B/Fichier B.txt"), "content").unwrap();

    let output = cmd().arg("-r").arg("-n").arg(dir).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("dossier-a"));
    assert!(stdout.contains("dossier-b"));
    assert!(stdout.contains("fichier-a.txt"));
    assert!(stdout.contains("fichier-b.txt"));
}

#[test]
fn test_recursive_with_hidden_subdirs_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join(".hidden")).unwrap();
    fs::write(dir.join("visible.txt"), "content").unwrap();

    let output = cmd().arg("-r").arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join(".hidden").exists());
    assert!(dir.join("visible.txt").exists());
}
