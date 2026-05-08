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

// ─────────────────────────────────────────────────────────────────────────────
// stdout / stderr discipline (quiet mode is the default)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quiet_mode_produces_no_stdout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(
        output.stdout.is_empty(),
        "expected no stdout in quiet mode, got: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(dir.join("fichier-test.txt").exists());
}

#[test]
fn test_quiet_mode_still_reports_conflicts_on_stderr() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "1").unwrap();
    fs::write(dir.join("CAFÉ.TXT"), "2").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_uppercase().contains("CONFLICT"));
}

#[test]
fn test_empty_directory_produces_no_output() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd().arg(temp_dir.path()).output().unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Argument validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_nonexistent_directory_is_an_error() {
    let output = cmd().arg("/this/path/does/not/exist/xyz").output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("not a directory")
            || stderr.to_lowercase().contains("error")
    );
}

#[test]
fn test_file_path_as_dir_argument_is_an_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("regular.txt");
    fs::write(&file_path, "x").unwrap();

    let output = cmd().arg(&file_path).output().unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_f_and_d_together_are_rejected() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg("-f")
        .arg("-d")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-f") && stderr.contains("-d"));
}

#[test]
fn test_multiple_positional_args_rejected() {
    let temp_dir1 = tempfile::tempdir().unwrap();
    let temp_dir2 = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg(temp_dir1.path())
        .arg(temp_dir2.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_unknown_flag_rejected() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg("--this-flag-does-not-exist")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lower = stderr.to_lowercase();
    // Accept any of the common wordings (our own "Unknown flag", clap's
    // "unexpected argument", or "unrecognized") so the test stays stable
    // across argument-parser changes.
    assert!(
        lower.contains("unknown") || lower.contains("unexpected") || lower.contains("unrecognized"),
        "stderr should explain that the flag is invalid, got: {stderr}"
    );
    // The bad flag itself must be echoed back to the user.
    assert!(stderr.contains("--this-flag-does-not-exist"));
}

#[test]
fn test_trailing_slash_argument_works() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();
    let dir_with_slash = format!("{}/", dir.display());

    let output = cmd().arg(&dir_with_slash).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier.txt").exists());
}

#[test]
fn test_flag_order_does_not_matter() {
    let make_dir = || {
        let td = tempfile::tempdir().unwrap();
        fs::write(td.path().join("Mon Fichier.txt"), "x").unwrap();
        fs::create_dir(td.path().join("Mon Dossier")).unwrap();
        td
    };

    let td1 = make_dir();
    let out1 = cmd()
        .arg("-r")
        .arg("-n")
        .arg("-v")
        .arg(td1.path())
        .output()
        .unwrap();

    let td2 = make_dir();
    let out2 = cmd()
        .arg("-v")
        .arg("-n")
        .arg("-r")
        .arg(td2.path())
        .output()
        .unwrap();

    assert!(out1.status.success() && out2.status.success());
    let stdout1 = String::from_utf8_lossy(&out1.stdout);
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    // Both should mention the same renames (order may differ across runs)
    assert!(stdout1.contains("mon-fichier.txt") && stdout2.contains("mon-fichier.txt"));
    assert!(stdout1.contains("mon-dossier") && stdout2.contains("mon-dossier"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Conflict handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_existing_destination_keeps_source_intact() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "source").unwrap();
    fs::write(dir.join("cafe.txt"), "destination").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("café.txt").exists(), "source must not be deleted");
    assert_eq!(fs::read_to_string(dir.join("café.txt")).unwrap(), "source");
    assert_eq!(
        fs::read_to_string(dir.join("cafe.txt")).unwrap(),
        "destination",
        "destination must not be overwritten"
    );
}

#[test]
fn test_three_way_conflict_all_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "1").unwrap();
    fs::write(dir.join("café.txt"), "2").unwrap();
    fs::write(dir.join("CAFE.txt"), "3").unwrap();

    let output = cmd().arg(dir).output().unwrap();

    assert!(output.status.success());
    // None of the three should have been collapsed into a single cafe.txt
    assert!(dir.join("Café.txt").exists());
    assert!(dir.join("café.txt").exists());
    assert!(dir.join("CAFE.txt").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_uppercase().contains("CONFLICT"));
}
