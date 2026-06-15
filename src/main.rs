use clap::{CommandFactory, Parser};
use rename_files::{compute_renames, RenameOp, RenameTarget};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

/// Lossy display name for a path, with a stable fallback when `file_name()`
/// is `None` (e.g. a path ending in `..`). Avoids the panicky `unwrap()` on
/// `file_name()` in the rename / conflict reporting code paths.
fn display_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .map_or(Cow::Borrowed("?"), |n| n.to_string_lossy())
}

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

/// Command-line interface, parsed by `clap`.
///
/// Mutual exclusion between `-f`, `-d`, and `-a` is enforced by
/// `conflicts_with_all`, so clap rejects invalid combinations at parse time.
#[derive(Parser, Debug)]
#[command(
    name = "rename-simple",
    version,
    about = "Rename files by normalising accented characters, spaces and \
             special chars to clean ASCII slugs",
    long_about = None,
)]
// Six independent flags is normal for a CLI; refactoring into a state
// enum would obscure the clap derive layout without simplifying the call
// sites.
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Rename files only
    #[arg(short = 'f', conflicts_with_all = ["dirs_only", "all"])]
    files_only: bool,

    /// Rename directories only
    #[arg(short = 'd', conflicts_with_all = ["files_only", "all"])]
    dirs_only: bool,

    /// Rename both files and directories
    #[arg(short = 'a', long, conflicts_with_all = ["files_only", "dirs_only"])]
    all: bool,

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
    let target = if cli.files_only {
        RenameTarget::FilesOnly
    } else if cli.dirs_only {
        RenameTarget::DirsOnly
    } else {
        RenameTarget::All
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
                display_name(&op.from),
                display_name(&op.to),
            );
        } else if op.to.exists() {
            eprintln!(
                "  ⚠  CONFLICT – skipping '{}': destination '{}' already exists",
                display_name(&op.from),
                display_name(&op.to),
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

/// Rename `from` to `to`, refusing to overwrite an existing destination.
///
/// Closes the TOCTOU window between the `op.to.exists()` pre-check in
/// `filter_conflicts` and the actual rename:
///
/// - **Linux**: `renameat2(AT_FDCWD, from, AT_FDCWD, to, RENAME_NOREPLACE)`
///   via `rustix` — atomic at the syscall level.
/// - **Other Unix / Windows**: a pre-check `try_exists` followed by
///   `std::fs::rename`. The race window remains in theory, but
///   `std::fs::rename` on Windows is already non-clobbering.
fn rename_no_clobber(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{renameat_with, RenameFlags, CWD};
        renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE).map_err(std::io::Error::from)
    }
    #[cfg(not(target_os = "linux"))]
    {
        if to.try_exists()? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "destination exists",
            ));
        }
        fs::rename(from, to)
    }
}

/// Apply a list of rename operations, printing each result.
/// Updates `counters` in place and returns the ops that were actually applied
/// to disk (empty in dry-run), so the caller can map each original path to its
/// real new location for recursion — instead of recomputing and probing the
/// filesystem, which is ambiguous when a sibling already holds the target name.
fn apply_ops(
    ops: &[RenameOp],
    dry_run: bool,
    verbose: bool,
    counters: &mut Counters,
) -> Vec<RenameOp> {
    let mut applied = Vec::new();
    for op in ops {
        let from_name = display_name(&op.from);
        let to_name = display_name(&op.to);

        if dry_run {
            if verbose {
                println!("  {from_name} → {to_name}");
            }
            counters.renamed += 1;
        } else {
            match rename_no_clobber(&op.from, &op.to) {
                Ok(()) => {
                    if verbose {
                        println!("  {from_name} → {to_name}");
                    }
                    counters.renamed += 1;
                    applied.push(op.clone());
                }
                Err(e) => {
                    eprintln!("  ✗ Error renaming '{from_name}': {e}");
                    counters.errors += 1;
                }
            }
        }
    }
    applied
}

/// Warn about entries whose filename is not valid UTF-8.
///
/// `compute_renames` skips these silently (it cannot transliterate bytes
/// it cannot decode). In verbose mode we want the user to know which
/// entries were ignored, otherwise a non-UTF-8 filename simply vanishes
/// from the report.
fn warn_invalid_utf8_entries(dir: &Path) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let name = entry.file_name();
        if name.to_str().is_none() {
            eprintln!(
                "  ⚠  Skipping entry with invalid UTF-8 name: '{}'",
                name.to_string_lossy()
            );
        }
    }
}

/// Collect non-hidden subdirectories inside `dir`.
/// Called BEFORE applying renames so we capture the original paths.
///
/// Symlinks pointing at directories are intentionally **not** followed:
/// `symlink_metadata` returns the link's own metadata, so its file type is
/// `Symlink`, not `Dir`. This prevents `-r` from escaping the target tree
/// or looping on cyclic links.
fn collect_subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    read_dir
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            fs::symlink_metadata(p)
                .map(|m| m.file_type().is_dir())
                .unwrap_or(false)
        })
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
/// - Otherwise, look the original path up in the set of renames that were
///   actually applied. A directory whose rename was skipped (conflict, error…)
///   is absent from the map and keeps its original path. This is unambiguous
///   even when a sibling already holds the target name — unlike recomputing the
///   destination name and probing the filesystem.
fn effective_subdir_path(
    original: &Path,
    applied: &HashMap<&Path, &Path>,
    dry_run: bool,
) -> PathBuf {
    if dry_run {
        return original.to_owned();
    }
    applied
        .get(original)
        .map_or_else(|| original.to_owned(), |p| p.to_path_buf())
}

/// Process a single directory: rename its entries, then optionally recurse.
///
/// The function prints a header for the directory, applies (or simulates) all
/// renames, and — when `config.recursive` is true — descends into every
/// subdirectory using its post-rename path.
fn process_dir(dir: &Path, config: &Config, counters: &mut Counters) {
    if config.verbose {
        println!("Directory: {}\n", dir.display());
        warn_invalid_utf8_entries(dir);
    }

    // Snapshot subdirs BEFORE any rename so we can resolve their new paths later.
    let subdirs = if config.recursive {
        collect_subdirs(dir)
    } else {
        Vec::new()
    };

    // Compute and apply renames at this level, keeping the ops that actually
    // landed on disk so recursion can follow each subdir to its real new path.
    let mut applied = Vec::new();
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
            applied = apply_ops(&ops, config.dry_run, config.verbose, counters);
            if config.verbose {
                println!();
            }
        }
    }

    // Recurse into subdirectories using their (potentially renamed) paths.
    if config.recursive {
        let applied_map: HashMap<&Path, &Path> = applied
            .iter()
            .map(|op| (op.from.as_path(), op.to.as_path()))
            .collect();
        for subdir in &subdirs {
            let effective = effective_subdir_path(subdir, &applied_map, config.dry_run);
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
    // `--version`, so anything reaching the next check already has a
    // syntactically valid command line.
    let cli = Cli::parse();

    // Without a target-mode flag (-f, -d, or -a), show help just like -h.
    if !cli.files_only && !cli.dirs_only && !cli.all {
        Cli::command().print_help().unwrap_or(());
        process::exit(0);
    }

    let config = match build_config(cli) {
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
