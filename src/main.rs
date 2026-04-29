use rename_files::{compute_renames, transform_filename, RenameOp, RenameTarget};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

fn usage(program: &str) {
    eprintln!("Usage: {program} [OPTIONS] [DIR]");
    eprintln!();
    eprintln!("Rename files and/or directories in DIR (default: current directory)");
    eprintln!("by normalising accented characters, replacing spaces and special");
    eprintln!("chars with `-`, and lowercasing everything.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -f              Rename files only (default: files + directories)");
    eprintln!("  -d              Rename directories only");
    eprintln!("  -r, --recursive Process subdirectories recursively");
    eprintln!("  -n, --dry-run   Show what would be renamed without touching any entry");
    eprintln!("  -h, --help      Print this help message");
}

struct Config {
    dir: PathBuf,
    dry_run: bool,
    recursive: bool,
    target: RenameTarget,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut dry_run = false;
    let mut recursive = false;
    let mut files_only = false;
    let mut dirs_only = false;
    let mut dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                usage(program);
                process::exit(0);
            }
            "-n" | "--dry-run" => {
                dry_run = true;
            }
            "-r" | "--recursive" => {
                recursive = true;
            }
            "-f" => {
                files_only = true;
            }
            "-d" => {
                dirs_only = true;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown flag: {flag}"));
            }
            path => {
                if dir.is_some() {
                    return Err("Too many positional arguments.".to_owned());
                }
                dir = Some(PathBuf::from(path));
            }
        }
        i += 1;
    }

    if files_only && dirs_only {
        return Err("-f and -d are mutually exclusive.".to_owned());
    }

    let target = match (files_only, dirs_only) {
        (true, false) => RenameTarget::FilesOnly,
        (false, true) => RenameTarget::DirsOnly,
        _ => RenameTarget::All,
    };

    let dir =
        dir.unwrap_or_else(|| env::current_dir().expect("Cannot determine current directory"));

    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory.", dir.display()));
    }

    Ok(Config {
        dir,
        dry_run,
        recursive,
        target,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Conflict detection
// ─────────────────────────────────────────────────────────────────────────────

/// Return ops that are safe to apply (no destination conflicts).
/// Conflicting ops are printed as warnings and dropped.
fn filter_conflicts(ops: Vec<RenameOp>) -> Vec<RenameOp> {
    let mut dest_count: HashMap<PathBuf, usize> = HashMap::new();
    for op in &ops {
        *dest_count.entry(op.to.clone()).or_insert(0) += 1;
    }

    let mut safe = Vec::new();
    for op in ops {
        if dest_count[&op.to] > 1 {
            eprintln!(
                "  ⚠  CONFLICT – skipping '{}': multiple entries would rename to '{}'",
                op.from.file_name().unwrap().to_string_lossy(),
                op.to.file_name().unwrap().to_string_lossy(),
            );
        } else if op.to.exists() {
            eprintln!(
                "  ⚠  CONFLICT – skipping '{}': destination '{}' already exists",
                op.from.file_name().unwrap().to_string_lossy(),
                op.to.file_name().unwrap().to_string_lossy(),
            );
        } else {
            safe.push(op);
        }
    }
    safe
}

// ─────────────────────────────────────────────────────────────────────────────
// Directory processing
// ─────────────────────────────────────────────────────────────────────────────

/// Counters accumulated across all levels of recursion.
struct Counters {
    renamed: usize,
    errors: usize,
}

/// Apply a list of rename operations, printing each result.
/// Updates `counters` in place.
fn apply_ops(ops: &[RenameOp], dry_run: bool, counters: &mut Counters) {
    for op in ops {
        let from_name = op.from.file_name().unwrap().to_string_lossy();
        let to_name = op.to.file_name().unwrap().to_string_lossy();

        if dry_run {
            println!("  {} → {}", from_name, to_name);
            counters.renamed += 1;
        } else {
            match fs::rename(&op.from, &op.to) {
                Ok(()) => {
                    println!("  {} → {}", from_name, to_name);
                    counters.renamed += 1;
                }
                Err(e) => {
                    eprintln!("  ✗ Error renaming '{}': {}", from_name, e);
                    counters.errors += 1;
                }
            }
        }
    }
}

/// Collect non-hidden subdirectories inside `dir`.
/// Called BEFORE applying renames so we capture the original paths.
fn collect_subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    read_dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('.'))
                .unwrap_or(false)
        })
        .collect()
}

/// Resolve the effective path of a subdirectory after the parent level has
/// been processed.
///
/// - In dry-run mode the filesystem was not modified, so the original path is
///   always valid.
/// - Otherwise, compute the expected new name; return it if it exists on disk,
///   fall back to the original if the rename was skipped (conflict, error…).
fn effective_subdir_path(original: &Path, parent: &Path, dry_run: bool) -> PathBuf {
    if dry_run {
        return original.to_owned();
    }
    if let Some(name) = original.file_name().and_then(|n| n.to_str()) {
        let new_name = transform_filename(name);
        let candidate = parent.join(&new_name);
        if candidate.is_dir() {
            return candidate;
        }
    }
    // Rename was skipped or failed — fall back to original path
    original.to_owned()
}

/// Process a single directory: rename its entries, then optionally recurse.
///
/// The function prints a header for the directory, applies (or simulates) all
/// renames, and — when `config.recursive` is true — descends into every
/// subdirectory using its post-rename path.
fn process_dir(dir: &Path, config: &Config, counters: &mut Counters) {
    println!("Directory: {}\n", dir.display());

    // Snapshot subdirs BEFORE any rename so we can resolve their new paths later.
    let subdirs = if config.recursive {
        collect_subdirs(dir)
    } else {
        Vec::new()
    };

    // Compute and apply renames at this level.
    match compute_renames(dir, config.target) {
        Err(e) => {
            eprintln!("  ✗ Cannot read directory: {e}");
        }
        Ok(ops) if ops.is_empty() => {
            println!("  (nothing to rename)\n");
        }
        Ok(ops) => {
            let ops = filter_conflicts(ops);
            apply_ops(&ops, config.dry_run, counters);
            println!();
        }
    }

    // Recurse into subdirectories using their (potentially renamed) paths.
    if config.recursive {
        for subdir in &subdirs {
            let effective = effective_subdir_path(subdir, dir, config.dry_run);
            if effective.is_dir() {
                process_dir(&effective, config, counters);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("Error: {msg}");
            eprintln!("Run with --help for usage.");
            process::exit(1);
        }
    };

    if config.dry_run {
        println!("Dry run – no files will be modified.\n");
    }

    let mut counters = Counters {
        renamed: 0,
        errors: 0,
    };

    process_dir(&config.dir, &config, &mut counters);

    let label = if config.dry_run {
        "would be renamed"
    } else {
        "renamed"
    };
    println!(
        "{} entry/entries {label}, {} error(s).",
        counters.renamed, counters.errors
    );
}
