use clap::{CommandFactory, Parser};
use rename_files::{plan_rename, RenameOp, RenameTarget};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

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
/// Mutual exclusion between `-f` and `-d` is enforced by `conflicts_with`, so
/// clap rejects invalid combinations at parse time.
#[derive(Parser, Debug)]
#[command(
    name = "rename-simple",
    version,
    about = "Rename files by normalising accented characters, spaces and \
             special chars to clean ASCII slugs",
    long_about = None,
)]
// Four independent flags is normal for a CLI; refactoring into a state enum
// would obscure the clap derive layout without simplifying the call sites.
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Rename files only
    #[arg(short = 'f', conflicts_with = "dirs_only")]
    files_only: bool,

    /// Rename directories only
    #[arg(short = 'd', conflicts_with = "files_only")]
    dirs_only: bool,

    /// Show details of what is being renamed
    #[arg(short, long)]
    verbose: bool,

    /// Show what would be renamed without touching any entry
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Entries to rename (files and/or directories). Each one is renamed
    /// itself, like the traditional `rename`(1) command. Globbing is left to
    /// the shell.
    #[arg(value_name = "files")]
    paths: Vec<PathBuf>,
}

struct Config {
    dry_run: bool,
    target: RenameTarget,
    verbose: bool,
}

/// Select what kind of entries to rename from the parsed flags. Without `-f`
/// or `-d` both files and directories are renamed.
fn target_from(cli: &Cli) -> RenameTarget {
    if cli.files_only {
        RenameTarget::FilesOnly
    } else if cli.dirs_only {
        RenameTarget::DirsOnly
    } else {
        RenameTarget::All
    }
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
// Renaming
// ─────────────────────────────────────────────────────────────────────────────

/// Counters accumulated across the whole batch.
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
        std::fs::rename(from, to)
    }
}

/// Apply a list of rename operations, printing each result.
/// Updates `counters` in place.
fn apply_ops(ops: &[RenameOp], dry_run: bool, verbose: bool, counters: &mut Counters) {
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
                }
                Err(e) => {
                    eprintln!("  ✗ Error renaming '{from_name}': {e}");
                    counters.errors += 1;
                }
            }
        }
    }
}

/// Rename a list of explicitly-named entries (the `rename`-like mode).
///
/// Each path is renamed **itself** (not its contents). Missing paths are
/// reported and counted as errors but do not abort the batch. Destination
/// conflicts across the whole batch are detected together via `filter_conflicts`.
fn process_targets(paths: &[PathBuf], config: &Config, counters: &mut Counters) {
    let mut ops = Vec::new();
    for path in paths {
        if !path.exists() {
            eprintln!("  ✗ Error: '{}' does not exist", path.display());
            counters.errors += 1;
            continue;
        }
        if let Some(op) = plan_rename(path, config.target) {
            ops.push(op);
        }
    }

    let ops = filter_conflicts(ops);
    apply_ops(&ops, config.dry_run, config.verbose, counters);
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    // `Cli::parse()` exits the process on parse errors and on `--help` /
    // `--version`, so anything reaching the next check already has a
    // syntactically valid command line.
    let cli = Cli::parse();

    // Without explicit paths there is nothing to operate on: show help like -h.
    if cli.paths.is_empty() {
        Cli::command().print_help().unwrap_or(());
        process::exit(0);
    }

    let config = Config {
        dry_run: cli.dry_run,
        target: target_from(&cli),
        verbose: cli.verbose,
    };

    if config.dry_run && config.verbose {
        println!("Dry run – no files will be modified.\n");
    }

    let mut counters = Counters {
        renamed: 0,
        errors: 0,
    };

    process_targets(&cli.paths, &config, &mut counters);

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
