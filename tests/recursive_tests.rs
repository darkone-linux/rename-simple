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

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

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

    let output = cmd()
        .arg("-a")
        .arg("-r")
        .arg("-n")
        .current_dir(dir)
        .output()
        .unwrap();

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

    let output = cmd().arg("-f").arg("-r").current_dir(dir).output().unwrap();

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

    let output = cmd().arg("-d").arg("-r").current_dir(dir).output().unwrap();

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

    let output = cmd()
        .arg("-a")
        .arg("-n")
        .arg("-v")
        .current_dir(dir)
        .output()
        .unwrap();

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

    let output = cmd()
        .arg("-a")
        .arg("-r")
        .arg("-n")
        .arg("-v")
        .current_dir(dir)
        .output()
        .unwrap();
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

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join(".hidden").exists());
    assert!(dir.join("visible.txt").exists());
}

// ─────────────────────────────────────────────────────────────────────────────
// Audit additions: stress + ordering guarantees
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_recursive_deep_nesting_terminates() {
    // 15 levels of nesting: catches accidental stack overflow or
    // combinatorial explosion in process_dir.
    let temp_dir = tempfile::tempdir().unwrap();
    let mut current = temp_dir.path().to_path_buf();
    let depth = 15usize;
    for i in 0..depth {
        current = current.join(format!("Niveau {i}"));
        fs::create_dir(&current).unwrap();
    }
    fs::write(current.join("Fichier Profond.txt"), "deep").unwrap();

    let output = cmd()
        .arg("-a")
        .arg("-r")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "must not crash at depth {depth}");

    // Walk the renamed tree end-to-end.
    let mut walked = temp_dir.path().to_path_buf();
    for i in 0..depth {
        walked = walked.join(format!("niveau-{i}"));
        assert!(walked.is_dir(), "missing renamed dir: {walked:?}");
    }
    assert!(walked.join("fichier-profond.txt").exists());
}

#[test]
fn test_recursive_descends_into_renamed_dir_not_original() {
    // After renaming "Mon Dossier" → "mon-dossier", recursion must follow
    // the NEW path. effective_subdir_path is the function under test here.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Mon Dossier")).unwrap();
    fs::write(dir.join("Mon Dossier/Fichier Interne.txt"), "x").unwrap();

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(!dir.join("Mon Dossier").exists(), "parent must be renamed");
    assert!(
        dir.join("mon-dossier/fichier-interne.txt").exists(),
        "child must be renamed inside the renamed parent"
    );
}

#[test]
fn test_recursive_sibling_conflict_descends_into_blocked_dir() {
    // Regression for the documented "recursion descends into the wrong sibling
    // on a name conflict" limitation.
    //
    //   parent/
    //   ├── Cible/                 ← would rename to "cible" but blocked
    //   │   └── Sous Fichier.txt   ← must still be visited inside Cible/
    //   └── cible/                 ← pre-existing, must NOT be visited twice
    //
    // Recursion must follow the *actual* on-disk path of "Cible" (unchanged,
    // because the rename was skipped), not the recomputed destination "cible".
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Cible")).unwrap();
    fs::write(dir.join("Cible/Sous Fichier.txt"), "x").unwrap();
    fs::create_dir(dir.join("cible")).unwrap();
    fs::write(dir.join("cible/Deja Propre.txt"), "y").unwrap();

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();
    assert!(output.status.success());

    // The blocked directory still exists under its original name and its
    // content was renamed in place.
    assert!(dir.join("Cible").exists(), "conflicting dir must stay");
    assert!(
        dir.join("Cible/sous-fichier.txt").exists(),
        "file inside the blocked dir must be renamed (recursion must visit it)"
    );

    // The pre-existing sibling is visited exactly once: its file is renamed,
    // and Cible's content did not leak into it.
    assert!(dir.join("cible/deja-propre.txt").exists());
    assert!(
        !dir.join("cible/sous-fichier.txt").exists(),
        "blocked dir content must not be processed inside the wrong sibling"
    );
}

#[test]
fn test_recursive_processes_sibling_subdirs_independently() {
    // Two parallel subdirectories, each renamed and each containing a file
    // that must also be renamed inside the new parent path.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Alpha Bêta")).unwrap();
    fs::create_dir(dir.join("Gamma Delta")).unwrap();
    fs::write(dir.join("Alpha Bêta/Un Fichier.txt"), "a").unwrap();
    fs::write(dir.join("Gamma Delta/Autre Fichier.txt"), "g").unwrap();

    let output = cmd().arg("-a").arg("-r").current_dir(dir).output().unwrap();

    assert!(output.status.success());
    assert!(dir.join("alpha-beta/un-fichier.txt").exists());
    assert!(dir.join("gamma-delta/autre-fichier.txt").exists());
}

// ─────────────────────────────────────────────────────────────────────────────
// Explicit directory target with -r (`rename`-like mode): rename + descend
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_explicit_dir_target_with_r_renames_and_descends() {
    // `rename-simple -r DIR` (DIR as an explicit argument) renames the directory
    // itself AND recursively cleans its contents.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Mon Dossier")).unwrap();
    fs::create_dir(dir.join("Mon Dossier/Sous Dossier")).unwrap();
    fs::write(dir.join("Mon Dossier/Fichier A.txt"), "a").unwrap();
    fs::write(dir.join("Mon Dossier/Sous Dossier/Fichier B.txt"), "b").unwrap();

    let output = cmd()
        .arg("-a")
        .arg("-r")
        .arg(dir.join("Mon Dossier"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        !dir.join("Mon Dossier").exists(),
        "target dir must be renamed"
    );
    assert!(dir.join("mon-dossier/fichier-a.txt").exists());
    assert!(dir.join("mon-dossier/sous-dossier/fichier-b.txt").exists());
}

#[test]
fn test_explicit_dir_target_without_r_renames_self_only() {
    // Without -r, an explicit directory target is renamed but its contents are
    // left untouched.
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    fs::create_dir(dir.join("Mon Dossier")).unwrap();
    fs::write(dir.join("Mon Dossier/Fichier A.txt"), "a").unwrap();

    let output = cmd()
        .arg("-a")
        .arg(dir.join("Mon Dossier"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        dir.join("mon-dossier").exists(),
        "target dir must be renamed"
    );
    assert!(
        dir.join("mon-dossier/Fichier A.txt").exists(),
        "contents must be untouched without -r"
    );
}
