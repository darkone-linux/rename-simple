use clap::Parser;
use rename_files::{compute_renames, transform_filename, RenameOp, RenameTarget};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

/// Command-line interface, parsed by `clap`.
///
/// Mutual exclusion between `-f` and `-d` is enforced by `conflicts_with`,
/// so clap rejects `rename-simple -f -d` at parse time with a message that
/// mentions both flags.
#[derive(Parser, Debug)]
#[command(
    name = "rename-simple",
    version,
    about = "Rename files by normalising accented characters, spaces and \
             special chars to clean ASCII slugs",
    long_about = None,
)]
// Five independent flags is normal for a CLI; refactoring into a state
// enum would obscure the clap derive layout without simplifying the call
// sites.
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Rename files only (default: files + directories)
    #[arg(short = 'f', conflicts_with = "dirs_only")]
    files_only: bool,

    /// Rename directories only
    #[arg(short = 'd')]
    dirs_only: bool,

    /// Process subdirectories recursively
    #[arg(short, long)]
    recursive: bool,

    /// Show details of what is being renamed
    #[arg(short, long)]
    verbose: bool,

    /// Show what would be renamed without touching any entry
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Target directory (default: current directory)
    dir: Option<PathBuf>,
}

struct Config {
    dir: PathBuf,
    dry_run: bool,
    recursive: bool,
    target: RenameTarget,
    verbose: bool,
}

/// Convert the parsed `Cli` into the runtime `Config`, performing the only
/// validation `clap` cannot express declaratively: that the target path is an
/// existing directory.
fn build_config(cli: Cli) -> Result<Config, String> {
    let target = match (cli.files_only, cli.dirs_only) {
        (true, false) => RenameTarget::FilesOnly,
        (false, true) => RenameTarget::DirsOnly,
        _ => RenameTarget::All,
    };

    let dir = cli.dir.map_or_else(
        || env::current_dir().map_err(|e| format!("Cannot determine current directory: {e}")),
        Ok,
    )?;

    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory.", dir.display()));
    }

    Ok(Config {
        dir,
        dry_run: cli.dry_run,
        recursive: cli.recursive,
        target,
        verbose: cli.verbose,
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
fn apply_ops(ops: &[RenameOp], dry_run: bool, verbose: bool, counters: &mut Counters) {
    for op in ops {
        let from_name = op.from.file_name().unwrap().to_string_lossy();
        let to_name = op.to.file_name().unwrap().to_string_lossy();

        if dry_run {
            if verbose {
                println!("  {from_name} → {to_name}");
            }
            counters.renamed += 1;
        } else {
            match fs::rename(&op.from, &op.to) {
                Ok(()) => {
                    if verbose {
                        println!("  {from_name} → {to_name}");
                    }
                    counters.renamed += 1;
                }
                Err(e) => {
                    eprintln!("  ✗ Error renaming '{from_name}': {e}");
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
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| !n.starts_with('.'))
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
    if config.verbose {
        println!("Directory: {}\n", dir.display());
    }

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
            if config.verbose {
                println!("  (nothing to rename)\n");
            }
        }
        Ok(ops) => {
            let ops = filter_conflicts(ops);
            apply_ops(&ops, config.dry_run, config.verbose, counters);
            if config.verbose {
                println!();
            }
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
    // `Cli::parse()` exits the process on parse errors and on `--help` /
    // `--version`, so anything reaching `build_config` already has a
    // syntactically valid command line.
    let config = match build_config(Cli::parse()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("Error: {msg}");
            eprintln!("Run with --help for usage.");
            process::exit(1);
        }
    };

    if config.dry_run && config.verbose {
        println!("Dry run – no files will be modified.\n");
    }

    let mut counters = Counters {
        renamed: 0,
        errors: 0,
    };

    process_dir(&config.dir, &config, &mut counters);

    if config.verbose {
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
}
