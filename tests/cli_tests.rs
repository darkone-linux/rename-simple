use assert_cmd::Command;
use std::fs;

fn cmd() -> Command {
    Command::cargo_bin("rename-simple").unwrap()
}

#[test]
fn test_renames_multiple_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();
    fs::write(dir.join("Café.md"), "content").unwrap();

    let output = cmd()
        .arg(dir.join("Fichier Test.txt"))
        .arg(dir.join("Café.md"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("fichier-test.txt").exists());
    assert!(dir.join("cafe.md").exists());
}

#[test]
fn test_renames_multiple_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Répertoire Test")).unwrap();
    fs::create_dir(dir.join("Café")).unwrap();

    let output = cmd()
        .arg(dir.join("Répertoire Test"))
        .arg(dir.join("Café"))
        .output()
        .unwrap();

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

    let output = cmd()
        .arg("-f")
        .arg(dir.join("Fichier.txt"))
        .arg(dir.join("Répertoire"))
        .output()
        .unwrap();

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

    let output = cmd()
        .arg("-d")
        .arg(dir.join("Fichier.txt"))
        .arg(dir.join("Répertoire"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!dir.join("fichier.txt").exists());
    assert!(dir.join("repertoire").exists());
}

#[test]
fn test_dry_run_no_actual_rename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    let file_path = dir.join("Fichier.txt");
    fs::write(&file_path, "content").unwrap();
    let original_content = fs::read(&file_path).unwrap();

    let output = cmd().arg("-n").arg(&file_path).output().unwrap();

    assert!(output.status.success());
    assert!(file_path.exists());
    assert!(!dir.join("fichier.txt").exists());
    assert_eq!(fs::read(&file_path).unwrap(), original_content);
}

#[test]
fn test_hidden_file_argument_is_left_alone() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join(".hidden"), "content").unwrap();
    fs::write(dir.join("Visible.txt"), "content").unwrap();

    let output = cmd()
        .arg(dir.join(".hidden"))
        .arg(dir.join("Visible.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join(".hidden").exists(), "hidden file must stay");
    assert!(dir.join("visible.txt").exists());
}

#[test]
fn test_conflict_warning() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "content1").unwrap();
    fs::write(dir.join("CAFÉ.TXT"), "content2").unwrap();

    let output = cmd()
        .arg("-n")
        .arg(dir.join("café.txt"))
        .arg(dir.join("CAFÉ.TXT"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("[E]"));
}

#[test]
fn test_existing_destination_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "content1").unwrap();
    fs::write(dir.join("cafe.txt"), "content2").unwrap();

    let output = cmd().arg("-n").arg(dir.join("café.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
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

    let output = cmd().arg(dir.join("Mon Fichier")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier").exists());
}

#[test]
fn test_compound_extension_preserved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Archive Test.tar.gz"), "content").unwrap();

    let output = cmd().arg(dir.join("Archive Test.tar.gz")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("archive-test.tar.gz").exists());
}

#[test]
fn test_numbers_preserved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("File 2024.txt"), "content").unwrap();

    let output = cmd().arg(dir.join("File 2024.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("file-2024.txt").exists());
}

// ─────────────────────────────────────────────────────────────────────────────
// Verbosity: quiet (-q) / normal (default) / verbose (-v)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quiet_mode_produces_no_output_at_all() {
    // -q suppresses everything: renamed lines, errors and the final report.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();

    let output = cmd()
        .arg("-q")
        .arg(dir.join("Fichier Test.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "quiet mode must be silent, got stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.join("fichier-test.txt").exists());
}

#[test]
fn test_quiet_mode_stays_silent_on_conflicts() {
    // Even conflicts are suppressed under -q.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "1").unwrap();
    fs::write(dir.join("CAFÉ.TXT"), "2").unwrap();

    let output = cmd()
        .arg("-q")
        .arg(dir.join("café.txt"))
        .arg(dir.join("CAFÉ.TXT"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

#[test]
fn test_normal_mode_reports_rename_and_summary() {
    // The default mode prints the [R] line on stdout and a final report.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Fichier Test.txt"), "content").unwrap();

    let output = cmd().arg(dir.join("Fichier Test.txt")).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[R]"), "expected a [R] line, got: {stdout}");
    assert!(stdout.contains("fichier-test.txt"));
    assert!(
        stdout.contains("1 entry matched") && stdout.contains("1 entry renamed"),
        "expected a summary line, got: {stdout}"
    );
}

#[test]
fn test_normal_mode_reports_conflicts_on_stderr() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("café.txt"), "1").unwrap();
    fs::write(dir.join("CAFÉ.TXT"), "2").unwrap();

    let output = cmd()
        .arg(dir.join("café.txt"))
        .arg(dir.join("CAFÉ.TXT"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
}

#[test]
fn test_noop_is_hidden_in_normal_but_shown_with_verbose() {
    // An already-clean target produces no [R]/[E] line. In normal mode only the
    // report shows it (as matched); with -v the [X] line appears too.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("already-clean.txt"), "content").unwrap();

    let normal = cmd().arg(dir.join("already-clean.txt")).output().unwrap();
    assert!(normal.status.success());
    let normal_out = String::from_utf8_lossy(&normal.stdout);
    assert!(!normal_out.contains("[X]"), "no [X] line without -v");
    assert!(normal_out.contains("1 entry matched") && normal_out.contains("0 entry renamed"));

    let verbose = cmd()
        .arg("-v")
        .arg(dir.join("already-clean.txt"))
        .output()
        .unwrap();
    assert!(verbose.status.success());
    let verbose_out = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose_out.contains("[X]"), "expected [X] line with -v");
    assert!(verbose_out.contains("already-clean.txt"));
}

#[test]
fn test_output_shows_directory_prefix() {
    // Each line is prefixed with the directory the entry lives in: `.` for the
    // current directory, and the relative sub-path for nested entries. The
    // prefix is rendered plain (uncolourised) when output is captured.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::create_dir(dir.join("Sub Dir")).unwrap();
    fs::write(dir.join("Sub Dir/Fichier À.txt"), "x").unwrap();

    // A relative argument so the prefix is exactly `Sub Dir`.
    let output = cmd()
        .current_dir(dir)
        .arg("Sub Dir/Fichier À.txt")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Sub Dir: Fichier À.txt -> fichier-a.txt"),
        "expected a directory-prefixed [R] line, got: {stdout}"
    );
}

#[test]
fn test_current_dir_prefix_is_dot() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();

    let output = cmd()
        .current_dir(dir)
        .arg("Mon Fichier.txt")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(".: Mon Fichier.txt -> mon-fichier.txt"),
        "expected a `.:` prefix for a current-dir entry, got: {stdout}"
    );
}

#[test]
fn test_quiet_conflicts_with_verbose() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .arg("-q")
        .arg("-v")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-q") && stderr.contains("-v"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Argument validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_nonexistent_path_reports_error() {
    // A non-existent explicit target is a per-entry error: reported on stderr,
    // and like any per-entry failure it sets the exit status to 1.
    let output = cmd().arg("/this/path/does/not/exist/xyz").output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "missing entry must be an error: {stderr}"
    );
}

#[test]
fn test_file_argument_is_renamed() {
    // Passing a file directly renames the file itself (the `rename`-like mode).
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let file_path = dir.join("Mon Fichier.txt");
    fs::write(&file_path, "x").unwrap();

    let output = cmd().arg(&file_path).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-fichier.txt").exists());
    assert!(!file_path.exists());
}

#[test]
fn test_no_arguments_shows_help() {
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

    let output = cmd().arg(&dir_with_slash).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("mon-dossier").exists());
    assert!(!dir.join("Mon Dossier").exists());
}

#[test]
fn test_flag_order_does_not_matter() {
    let make_target = || {
        let td = tempfile::tempdir().unwrap();
        fs::write(td.path().join("Mon Fichier.txt"), "x").unwrap();
        td
    };

    let td1 = make_target();
    let out1 = cmd()
        .arg("-n")
        .arg("-v")
        .arg(td1.path().join("Mon Fichier.txt"))
        .output()
        .unwrap();

    let td2 = make_target();
    let out2 = cmd()
        .arg("-v")
        .arg("-n")
        .arg(td2.path().join("Mon Fichier.txt"))
        .output()
        .unwrap();

    assert!(out1.status.success() && out2.status.success());
    let stdout1 = String::from_utf8_lossy(&out1.stdout);
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    assert!(stdout1.contains("mon-fichier.txt") && stdout2.contains("mon-fichier.txt"));
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

    let output = cmd().arg(dir.join("café.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
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

    let output = cmd()
        .arg(dir.join("!!!.txt"))
        .arg(dir.join("***.txt"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(dir.join("!!!.txt").exists(), "first source must stay");
    assert!(dir.join("***.txt").exists(), "second source must stay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[E]"));
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

    let output = cmd().arg(dir.join("!!!.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
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

    let output = cmd().arg("-n").arg(dir.join("Café.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
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

    let output = cmd()
        .arg(dir.join("Café.txt"))
        .arg(dir.join("café.txt"))
        .arg(dir.join("CAFE.txt"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    // None of the three should have been collapsed into a single cafe.txt
    assert!(dir.join("Café.txt").exists());
    assert!(dir.join("café.txt").exists());
    assert!(dir.join("CAFE.txt").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[E]"));
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
        .arg(dir.join("Café.txt"))
        .arg(dir.join("café.txt"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(dir.join("Café.txt").exists());
    assert!(dir.join("café.txt").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[E]"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Cleanup fixes: -U / --fix-unicode, -H / --fix-html, -A / --fix-all
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_fix_unicode_repairs_mojibake() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("CafÃ© MontrÃ©al.txt"), "content").unwrap();

    let output = cmd()
        .arg("-U")
        .arg(dir.join("CafÃ© MontrÃ©al.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("cafe-montreal.txt").exists());
}

#[test]
fn test_without_fix_unicode_mojibake_goes_through_raw_pipeline() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("CafÃ©.txt"), "content").unwrap();

    let output = cmd().arg(dir.join("CafÃ©.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("cafa.txt").exists());
}

#[test]
fn test_fix_html_strips_tags_and_entities() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    // Note: a real filename cannot contain '/', so closing tags like </b>
    // never appear; an opening tag and entities are the realistic case.
    fs::write(dir.join("<b>Tom &amp; Jerry.mp4"), "content").unwrap();

    let output = cmd()
        .arg("--fix-html")
        .arg(dir.join("<b>Tom &amp; Jerry.mp4"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("tom-jerry.mp4").exists());
}

#[test]
fn test_fix_html_decoded_slash_stays_in_the_basename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    // `&#47;` decodes to '/': the destination must stay a plain basename.
    fs::write(dir.join("foo.txt&#47;"), "content").unwrap();
    fs::write(dir.join("Report &#47; Final.txt"), "content").unwrap();

    let output = cmd()
        .arg("-H")
        .arg(dir.join("foo.txt&#47;"))
        .arg(dir.join("Report &#47; Final.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("foo-txt").exists());
    assert!(dir.join("report-final.txt").exists());
}

#[test]
fn test_fix_all_applies_unicode_and_html() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("CafÃ© &amp; <i>The.txt"), "content").unwrap();

    let output = cmd()
        .arg("-A")
        .arg(dir.join("CafÃ© &amp; <i>The.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("cafe-the.txt").exists());
}

#[test]
fn test_fix_all_on_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::create_dir(dir.join("DonnÃ©es &amp; Archives")).unwrap();

    let output = cmd()
        .arg("--fix-all")
        .arg(dir.join("DonnÃ©es &amp; Archives"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("donnees-archives").is_dir());
}

#[test]
fn test_cleanup_flags_combine_with_dry_run() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("CafÃ©.txt"), "content").unwrap();

    let output = cmd()
        .arg("-n")
        .arg("-U")
        .arg(dir.join("CafÃ©.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cafe.txt"),
        "dry-run must preview the fixed name: {stdout}"
    );
    assert!(dir.join("CafÃ©.txt").exists(), "dry-run must not rename");
}

// ─────────────────────────────────────────────────────────────────────────────
// Duplicate handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_identical_duplicate_source_is_deleted() {
    // The destination already exists but holds exactly the same bytes: the
    // source is a strict duplicate and gets removed instead of erroring out.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "same bytes").unwrap();
    fs::write(dir.join("cafe.txt"), "same bytes").unwrap();

    let output = cmd().arg("-D").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(
        !dir.join("Café.txt").exists(),
        "the duplicate source must be deleted"
    );
    assert_eq!(
        fs::read_to_string(dir.join("cafe.txt")).unwrap(),
        "same bytes",
        "the destination must be left untouched"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[W]"), "expected a [W] line, got: {stderr}");
    assert!(
        !stderr.contains("[E]"),
        "a strict duplicate is not an error: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1 duplicate removed"),
        "expected the duplicate count in the summary, got: {stdout}"
    );
}

#[test]
fn test_identical_empty_files_are_duplicates() {
    // Zero-length files compare equal: the size fast-path must not mistake
    // "nothing to read" for "contents differ".
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Vide Test.txt"), "").unwrap();
    fs::write(dir.join("vide-test.txt"), "").unwrap();

    let output = cmd()
        .arg("-D")
        .arg(dir.join("Vide Test.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!dir.join("Vide Test.txt").exists());
    assert!(dir.join("vide-test.txt").exists());
}

#[test]
fn test_same_size_different_content_is_an_error() {
    // Same length, different bytes: the comparison must go past the size
    // fast-path and keep both files.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "aaaa").unwrap();
    fs::write(dir.join("cafe.txt"), "bbbb").unwrap();

    let output = cmd().arg("-D").arg(dir.join("Café.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(dir.join("Café.txt").exists(), "source must stay put");
    assert_eq!(fs::read_to_string(dir.join("cafe.txt")).unwrap(), "bbbb");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
    assert!(!stderr.contains("[W]"));
}

#[test]
fn test_duplicate_dry_run_keeps_source() {
    // -n must not delete anything, only announce what it would do.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "same bytes").unwrap();
    fs::write(dir.join("cafe.txt"), "same bytes").unwrap();

    let output = cmd().arg("-nD").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(
        dir.join("Café.txt").exists(),
        "dry-run must keep the duplicate source"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[W]"), "expected a [W] line, got: {stderr}");
}

#[test]
fn test_duplicate_stays_silent_under_quiet() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "same bytes").unwrap();
    fs::write(dir.join("cafe.txt"), "same bytes").unwrap();

    let output = cmd().arg("-qD").arg(dir.join("Café.txt")).output().unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(
        !dir.join("Café.txt").exists(),
        "-q still deletes duplicates"
    );
}

#[test]
fn test_directory_destination_is_never_deleted() {
    // Directories are not comparable byte for byte: an existing directory at
    // the destination stays a plain error, whatever it contains.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Mon Dossier")).unwrap();
    fs::create_dir(dir.join("mon-dossier")).unwrap();

    let output = cmd()
        .arg("-D")
        .arg(dir.join("Mon Dossier"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(dir.join("Mon Dossier").exists(), "source dir must stay");
    assert!(dir.join("mon-dossier").exists(), "target dir must stay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
}

#[test]
fn test_duplicate_needs_the_flag_to_be_deleted() {
    // Without -D the strict duplicate is only reported: nothing is deleted,
    // and the message names the flag that would clear the clash.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("Café.txt"), "same bytes").unwrap();
    fs::write(dir.join("cafe.txt"), "same bytes").unwrap();

    let output = cmd().arg(dir.join("Café.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        dir.join("Café.txt").exists(),
        "no deletion without an explicit -D"
    );
    assert!(dir.join("cafe.txt").exists());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
    assert!(
        stderr.contains("identical duplicate") && stderr.contains("-D"),
        "the error should point at -D, got: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("duplicate removed"),
        "nothing was removed: {stdout}"
    );
}

#[test]
fn test_fix_all_does_not_delete_duplicates() {
    // -A is a name-repair shortcut only: it must never imply -D.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("CafÃ©.txt"), "same bytes").unwrap();
    fs::write(dir.join("cafe.txt"), "same bytes").unwrap();

    let output = cmd().arg("-A").arg(dir.join("CafÃ©.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        dir.join("CafÃ©.txt").exists(),
        "-A must not delete anything"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[E]"),
        "expected an [E] line, got: {stderr}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// File names that look like options (`rename-simple *` meeting a file `-D`)
// ─────────────────────────────────────────────────────────────────────────────

/// A directory where `rename-simple *` is dangerous: the glob hands the file
/// named `-D` to the program, which would read it as the option and delete
/// `IMG 1.mov` as a duplicate of `img-1.mov`.
fn dir_with_option_named_file() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("-D"), "").unwrap();
    fs::write(dir.join("IMG 1.mov"), "same bytes").unwrap();
    fs::write(dir.join("img-1.mov"), "same bytes").unwrap();
    temp_dir
}

#[test]
fn test_option_named_like_a_file_is_refused() {
    // What the shell passes for `rename-simple *` in that directory.
    let temp_dir = dir_with_option_named_file();
    let dir = temp_dir.path();

    let output = cmd()
        .args(["-D", "IMG 1.mov", "img-1.mov"])
        .current_dir(dir)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "an ambiguous command line is a usage error"
    );
    assert!(dir.join("IMG 1.mov").exists(), "nothing may be deleted");
    assert!(dir.join("-D").exists(), "nothing may be renamed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("\"-D\"") && stderr.contains("'--'"),
        "the error must name the argument and the `--` remedy, got: {stderr}"
    );
}

#[test]
fn test_long_option_named_like_a_file_is_refused() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("--delete-duplicates"), "").unwrap();
    fs::write(dir.join("IMG 1.mov"), "same bytes").unwrap();
    fs::write(dir.join("img-1.mov"), "same bytes").unwrap();

    let output = cmd()
        .args(["--delete-duplicates", "IMG 1.mov", "img-1.mov"])
        .current_dir(dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(dir.join("IMG 1.mov").exists(), "nothing may be deleted");
}

#[test]
fn test_lone_option_named_like_a_file_is_refused_even_quiet() {
    // `rename-simple *` in a directory holding only a file named `-q`: the
    // refusal must be reported, not swallowed by the very option it names.
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("-q"), "").unwrap();

    let output = cmd()
        .arg("-q")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("\"-q\""),
        "the refusal must be printed"
    );
}

#[test]
fn test_option_named_file_after_separator_is_a_file() {
    // `rename-simple -- *`: after `--` every argument is a file name.
    let temp_dir = dir_with_option_named_file();
    let dir = temp_dir.path();

    let output = cmd()
        .args(["--", "-D", "IMG 1.mov", "img-1.mov"])
        .current_dir(dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(dir.join("d").exists(), "`-D` is renamed like any file");
    assert!(
        dir.join("IMG 1.mov").exists(),
        "-D was not given: the duplicate is only reported"
    );
}

#[test]
fn test_typed_option_before_separator_is_honoured() {
    // With `--` the options before it are trusted, even when a file of the
    // same name exists: `rename-simple -n -- *` stays a dry run.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("-n"), "").unwrap();
    fs::write(dir.join("Mon Fichier.txt"), "x").unwrap();

    let output = cmd()
        .args(["-n", "--", "Mon Fichier.txt"])
        .current_dir(dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("Mon Fichier.txt").exists(), "-n must be honoured");
    assert!(String::from_utf8_lossy(&output.stdout).contains("mon-fichier.txt"));
}

#[test]
fn test_dot_slash_glob_never_reads_names_as_options() {
    // `rename-simple ./*`: no expanded name can start with `-`.
    let temp_dir = dir_with_option_named_file();
    let dir = temp_dir.path();

    let output = cmd()
        .args(["./-D", "./IMG 1.mov", "./img-1.mov"])
        .current_dir(dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(dir.join("d").exists());
    assert!(dir.join("IMG 1.mov").exists(), "nothing may be deleted");
}

// ─────────────────────────────────────────────────────────────────────────────
// One path, several spellings (overlapping globs, `./`, `..`)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_entry_given_several_times_is_renamed_once() {
    // Overlapping globs (`*.pdf Report*`) hand over the same entry twice.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("Report A.pdf"), "x").unwrap();

    let output = cmd()
        .current_dir(dir)
        .args([
            "Report A.pdf",
            "Report A.pdf",
            "./Report A.pdf",
            "sub/../Report A.pdf",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "no error expected, got: {stderr}");
    assert!(dir.join("report-a.pdf").exists());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1 entry matched, 1 entry renamed, 0 error."),
        "got: {stdout}"
    );
}

#[test]
fn test_two_spellings_of_one_destination_conflict() {
    // Two entries that both clean up to `note-b.txt`, handed over through
    // paths spelled differently: still a conflict, in the dry-run too.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Note B.txt"), "one").unwrap();
    fs::write(dir.join("Note  B.txt"), "two").unwrap();

    for flags in [&["-n"][..], &[]] {
        let output = cmd()
            .current_dir(dir)
            .args(flags)
            .args(["Note B.txt", "./Note  B.txt"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stdout.contains("[R]"), "{flags:?}: no rename: {stdout}");
        assert_eq!(
            stderr
                .matches("Multiple entries would produce this name")
                .count(),
            2,
            "{flags:?}: both entries are in conflict: {stderr}"
        );
    }
    assert_eq!(fs::read_to_string(dir.join("Note B.txt")).unwrap(), "one");
    assert_eq!(fs::read_to_string(dir.join("Note  B.txt")).unwrap(), "two");
    assert!(!dir.join("note-b.txt").exists());
}

#[test]
fn test_child_is_renamed_before_its_parent_however_spelled() {
    // The absolute path of the parent has more components than the relative
    // path of its child: renaming the parent first would leave the child's
    // path dangling.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::create_dir(dir.join("My Dir")).unwrap();
    fs::write(dir.join("My Dir").join("File A"), "x").unwrap();

    let output = cmd()
        .current_dir(dir)
        .arg(dir.join("My Dir"))
        .arg("My Dir/File A")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "no error expected, got: {stderr}");
    assert!(dir.join("my-dir").join("file-a").exists());
}

// ─────────────────────────────────────────────────────────────────────────────
// Exit status: 0 every entry handled, 1 an entry failed, 2 invalid command line
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_exit_status_is_0_when_every_entry_is_handled() {
    // Renamed, already clean, duplicate removed: nothing failed.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "a").unwrap();
    fs::write(dir.join("clean.txt"), "b").unwrap();
    fs::write(dir.join("Café.txt"), "same bytes").unwrap();
    fs::write(dir.join("cafe.txt"), "same bytes").unwrap();

    let output = cmd()
        .arg("-D")
        .arg(dir.join("Mon Fichier.txt"))
        .arg(dir.join("clean.txt"))
        .arg(dir.join("Café.txt"))
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_exit_status_is_1_when_an_entry_fails() {
    // Also in a dry-run, which predicts the real run, and under -q, which
    // leaves the status as the only signal. The other entries are processed
    // all the same.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Mon Fichier.txt"), "a").unwrap();

    for flags in [&["-n"][..], &["-q"], &[]] {
        let output = cmd()
            .args(flags)
            .arg(dir.join("Missing File.txt"))
            .arg(dir.join("Mon Fichier.txt"))
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1), "{flags:?}");
    }
    assert!(dir.join("mon-fichier.txt").exists());
}

#[test]
fn test_exit_status_is_1_on_a_conflict() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    fs::write(dir.join("Café.txt"), "one").unwrap();
    fs::write(dir.join("cafe.txt"), "two").unwrap();

    let output = cmd().arg(dir.join("Café.txt")).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
}
