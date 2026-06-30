use assert_cmd::Command;
use std::fs;

fn cmd() -> Command {
    Command::cargo_bin("rename-simple").unwrap()
}

#[test]
fn test_all_flag_renames_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();
    fs::write(dir.join("Café.md"), "content").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("fichier-test.txt").exists());
    assert!(dir.join("cafe.md").exists());
}

#[test]
fn test_all_flag_renames_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Répertoire Test")).unwrap();
    fs::create_dir(dir.join("Café")).unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-f").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-d").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").arg("-n").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").arg("-n").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").arg("-n").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier").exists());
}

#[test]
fn test_compound_extension_preserved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("archive.tar.gz"), "content").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("archive.tar.gz").exists());
}

#[test]
fn test_numbers_preserved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("File 2024.txt"), "content").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_uppercase().contains("CONFLICT"));
}

#[test]
fn test_empty_directory_produces_no_output() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg("-a")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Argument validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_nonexistent_path_reports_error() {
    // A non-existent explicit target is a per-entry error: reported on stderr
    // but, like any per-entry failure, it does not change the exit status.
    let output = cmd()
        .arg("-a")
        .arg("/this/path/does/not/exist/xyz")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("does not exist") || stderr.to_lowercase().contains("error")
    );
}

#[test]
fn test_file_argument_is_renamed() {
    // Passing a file directly renames the file itself (the `rename`-like mode).
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let file_path = dir.join("Mon Fichier.txt");
    fs::write(&file_path, "x").unwrap();

    let output = cmd().arg("-a").arg(&file_path).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier.txt").exists());
    assert!(!file_path.exists());
}

#[test]
fn test_no_target_flag_shows_help() {
    let output = cmd().output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_version_flag() {
    let output = cmd().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rename-simple"));
}

#[test]
fn test_a_and_f_together_are_rejected() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = cmd()
        .arg("-a")
        .arg("-f")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-a") && stderr.contains("-f"));
}

#[test]
fn test_a_and_d_together_are_rejected() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = cmd()
        .arg("-a")
        .arg("-d")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-a") && stderr.contains("-d"));
}

#[test]
fn test_f_and_d_together_are_rejected() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg("-f")
        .arg("-d")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-f") && stderr.contains("-d"));
}

#[test]
fn test_multiple_file_arguments_are_renamed() {
    // Several explicit targets at once: each is renamed independently.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Premier Fichier.txt"), "a").unwrap();
    fs::write(dir.join("Deuxième Fichier.txt"), "b").unwrap();

    let output = cmd()
        .arg("-a")
        .arg(dir.join("Premier Fichier.txt"))
        .arg(dir.join("Deuxième Fichier.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("premier-fichier.txt").exists());
    assert!(dir.join("deuxieme-fichier.txt").exists());
}

#[test]
fn test_unknown_flag_rejected() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg("--this-flag-does-not-exist")
        .current_dir(temp_dir.path())
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
    // A directory target given with a trailing slash is renamed itself: the
    // trailing slash must not confuse basename extraction in plan_rename.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Mon Dossier")).unwrap();
    let dir_with_slash = format!("{}/", dir.join("Mon Dossier").display());

    let output = cmd().arg("-a").arg(&dir_with_slash).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-dossier").exists());
    assert!(!dir.join("Mon Dossier").exists());
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
        .arg("-a")
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
        .arg("-a")
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

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

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
fn test_unnamed_collision_two_sources_skipped() {
    // Two filenames that both transliterate to "unnamed.txt" must NOT be
    // collapsed into a single file; filter_conflicts must detect the
    // duplicate destination and skip them.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("!!!.txt"), "1").unwrap();
    fs::write(dir.join("***.txt"), "2").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("!!!.txt").exists(), "first source must stay");
    assert!(dir.join("***.txt").exists(), "second source must stay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_uppercase().contains("CONFLICT"));
}

#[test]
fn test_unnamed_collides_with_existing_destination() {
    // A single non-alpha-num source name would map to "unnamed.txt"; when
    // that destination is already taken, the source must be left in place
    // and the existing file must NOT be overwritten.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("!!!.txt"), "source").unwrap();
    fs::write(dir.join("unnamed.txt"), "destination").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("!!!.txt").exists(), "source must stay put");
    assert_eq!(
        fs::read_to_string(dir.join("unnamed.txt")).unwrap(),
        "destination",
        "existing file must not be overwritten"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_uppercase().contains("CONFLICT") || stderr.to_uppercase().contains("EXISTS"),
        "stderr should report the conflict: {stderr}"
    );
}

#[test]
fn test_existing_destination_preserved_under_dry_run_too() {
    // Dry-run must not move data either: catch any future regression where
    // the dry-run path accidentally invokes rename. Combined with the
    // non-clobber rename helper, this guards both code paths.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "source").unwrap();
    fs::write(dir.join("cafe.txt"), "destination").unwrap();

    let output = cmd().arg("-a").arg("-n").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("Café.txt").exists());
    assert_eq!(fs::read_to_string(dir.join("Café.txt")).unwrap(), "source");
    assert_eq!(
        fs::read_to_string(dir.join("cafe.txt")).unwrap(),
        "destination"
    );
}

#[test]
fn test_three_way_conflict_all_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "1").unwrap();
    fs::write(dir.join("café.txt"), "2").unwrap();
    fs::write(dir.join("CAFE.txt"), "3").unwrap();

    let output = cmd().arg("-a").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    // None of the three should have been collapsed into a single cafe.txt
    assert!(dir.join("Café.txt").exists());
    assert!(dir.join("café.txt").exists());
    assert!(dir.join("CAFE.txt").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_uppercase().contains("CONFLICT"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Explicit targets (`rename`-like mode): type filters, dry-run, conflicts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_files_only_filter_skips_dir_argument() {
    // -f restricts explicit targets to files: a directory argument is left alone.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();
    fs::create_dir(dir.join("Mon Dossier")).unwrap();

    let output = cmd()
        .arg("-f")
        .arg(dir.join("Mon Fichier.txt"))
        .arg(dir.join("Mon Dossier"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier.txt").exists());
    assert!(
        dir.join("Mon Dossier").exists(),
        "dir must be skipped by -f"
    );
}

#[test]
fn test_dirs_only_filter_skips_file_argument() {
    // -d restricts explicit targets to directories: a file argument is left alone.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();
    fs::create_dir(dir.join("Mon Dossier")).unwrap();

    let output = cmd()
        .arg("-d")
        .arg(dir.join("Mon Fichier.txt"))
        .arg(dir.join("Mon Dossier"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-dossier").exists());
    assert!(
        dir.join("Mon Fichier.txt").exists(),
        "file must be skipped by -d"
    );
}

#[test]
fn test_mixed_file_and_dir_arguments_each_renamed() {
    // Without a type flag, both a file and a directory target are renamed.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();
    fs::create_dir(dir.join("Mon Dossier")).unwrap();

    let output = cmd()
        .arg(dir.join("Mon Fichier.txt"))
        .arg(dir.join("Mon Dossier"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier.txt").exists());
    assert!(dir.join("mon-dossier").exists());
}

#[test]
fn test_dry_run_on_explicit_file_touches_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let file_path = dir.join("Mon Fichier.txt");
    fs::write(&file_path, "x").unwrap();

    let output = cmd().arg("-n").arg(&file_path).output().unwrap();

    assert!(output.status.success());
    assert!(file_path.exists(), "original must be untouched in dry-run");
    assert!(!dir.join("mon-fichier.txt").exists());
}

#[test]
fn test_conflicting_explicit_arguments_are_skipped() {
    // Two arguments that would both collapse to the same destination conflict
    // with each other and must both be skipped with a warning.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Café.txt"), "1").unwrap();
    fs::write(dir.join("café.txt"), "2").unwrap();

    let output = cmd()
        .arg("-a")
        .arg(dir.join("Café.txt"))
        .arg(dir.join("café.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("Café.txt").exists());
    assert!(dir.join("café.txt").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_uppercase().contains("CONFLICT"));
}
