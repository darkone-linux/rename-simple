use clap::{CommandFactory, Parser};
use rename_files::{plan_entry, RenameOp, RenamePlan, RenameTarget};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::IsTerminal;
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
/// Mutual exclusion between `-f` and `-d`, and between `-q` and `-v`, is
/// enforced by `conflicts_with`, so clap rejects invalid combinations at parse
/// time.
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

    /// Print nothing at all
    #[arg(short = 'q', long, conflicts_with = "verbose")]
    quiet: bool,

    /// Show every entry, including the ones left untouched
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

/// How much the program prints on the standard streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verbosity {
    /// `-q`: print nothing at all.
    Quiet,
    /// Default: print renamed (`[R]`) and error (`[E]`) lines plus the report.
    Normal,
    /// `-v`: also print the untouched (`[X]`) lines.
    Verbose,
}

struct Config {
    dry_run: bool,
    target: RenameTarget,
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

/// Derive the verbosity level from the parsed flags. `-q` and `-v` are mutually
/// exclusive at the clap level, so at most one is set here.
fn verbosity_from(cli: &Cli) -> Verbosity {
    if cli.quiet {
        Verbosity::Quiet
    } else if cli.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Reporting
// ─────────────────────────────────────────────────────────────────────────────

const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

/// Wrap `text` in an ANSI colour escape, but only when `enabled` (the target
/// stream is a terminal). Piped / captured output stays plain.
fn paint(enabled: bool, code: &str, text: &str) -> String {
    if enabled {
        format!("{code}{text}{RESET}")
    } else {
        text.to_owned()
    }
}

/// "entry" for 0 or 1, "entries" for more than one.
fn plural_entries(n: usize) -> &'static str {
    if n > 1 {
        "entries"
    } else {
        "entry"
    }
}

/// "error" for 0 or 1, "errors" for more than one.
fn plural_errors(n: usize) -> &'static str {
    if n > 1 {
        "errors"
    } else {
        "error"
    }
}

/// Prints the per-entry lines and the final report, honouring the verbosity
/// level and disabling colours when the streams are not terminals.
struct Reporter {
    verbosity: Verbosity,
    stdout_tty: bool,
    stderr_tty: bool,
    renamed: usize,
    errors: usize,
    skipped: usize,
}

impl Reporter {
    fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            stdout_tty: std::io::stdout().is_terminal(),
            stderr_tty: std::io::stderr().is_terminal(),
            renamed: 0,
            errors: 0,
            skipped: 0,
        }
    }

    /// Report a successful (or, in dry-run, planned) rename: `[R] from -> to`.
    fn renamed(&mut self, from: &str, to: &str) {
        self.renamed += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stdout_tty, MAGENTA, "R");
        let arrow = paint(self.stdout_tty, MAGENTA, "->");
        println!("[{mark}] {from} {arrow} {to}");
    }

    /// Report a per-entry error: `[E] source -> message`.
    fn error(&mut self, source: &str, message: &str) {
        self.errors += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stderr_tty, RED, "E");
        let arrow = paint(self.stderr_tty, RED, "->");
        eprintln!("[{mark}] {source} {arrow} {message}");
    }

    /// Report an entry that matched but needed no rename: `[X] source`.
    /// Only shown in verbose mode.
    fn skipped(&mut self, source: &str) {
        self.skipped += 1;
        if self.verbosity != Verbosity::Verbose {
            return;
        }
        let mark = paint(self.stdout_tty, GREEN, "X");
        println!("[{mark}] {source}");
    }

    /// Print the final one-line summary. Silent in quiet mode.
    fn report(&self) {
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let matched = self.renamed + self.errors + self.skipped;
        let matched_word = plural_entries(matched);
        let renamed_word = plural_entries(self.renamed);
        let errors_word = plural_errors(self.errors);
        println!(
            "{matched} {matched_word} matched, {} {renamed_word} renamed, {} {errors_word}.",
            self.renamed, self.errors
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Conflict detection
// ─────────────────────────────────────────────────────────────────────────────

/// Return ops that are safe to apply (no destination conflicts).
/// Conflicting ops are reported as errors and dropped.
fn filter_conflicts(ops: Vec<RenameOp>, reporter: &mut Reporter) -> Vec<RenameOp> {
    let mut dest_count: HashMap<PathBuf, usize> = HashMap::new();
    for op in &ops {
        *dest_count.entry(op.to.clone()).or_insert(0) += 1;
    }

    let mut safe = Vec::new();
    for op in ops {
        if dest_count[&op.to] > 1 {
            reporter.error(
                &display_name(&op.from),
                "Multiple entries would produce this name",
            );
        } else if op.to.exists() {
            reporter.error(&display_name(&op.from), "File name already exists");
        } else {
            safe.push(op);
        }
    }
    safe
}

// ─────────────────────────────────────────────────────────────────────────────
// Renaming
// ─────────────────────────────────────────────────────────────────────────────

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

/// Apply a list of rename operations, reporting each result.
fn apply_ops(ops: &[RenameOp], dry_run: bool, reporter: &mut Reporter) {
    for op in ops {
        let from_name = display_name(&op.from);
        let to_name = display_name(&op.to);

        if dry_run {
            reporter.renamed(&from_name, &to_name);
        } else {
            match rename_no_clobber(&op.from, &op.to) {
                Ok(()) => reporter.renamed(&from_name, &to_name),
                Err(e) => reporter.error(&from_name, &e.to_string()),
            }
        }
    }
}

/// Rename a list of explicitly-named entries (the `rename`-like mode).
///
/// Each path is renamed **itself** (not its contents). Missing paths are
/// reported and counted as errors but do not abort the batch. Destination
/// conflicts across the whole batch are detected together via `filter_conflicts`.
fn process_targets(paths: &[PathBuf], config: &Config, reporter: &mut Reporter) {
    let mut ops = Vec::new();
    for path in paths {
        if !path.exists() {
            reporter.error(&path.display().to_string(), "No such file or directory");
            continue;
        }
        match plan_entry(path, config.target) {
            RenamePlan::Rename(op) => ops.push(op),
            RenamePlan::AlreadyClean => reporter.skipped(&display_name(path)),
            RenamePlan::Excluded => {}
        }
    }

    let ops = filter_conflicts(ops, reporter);
    apply_ops(&ops, config.dry_run, reporter);
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
    };

    let mut reporter = Reporter::new(verbosity_from(&cli));
    process_targets(&cli.paths, &config, &mut reporter);
    reporter.report();
}
