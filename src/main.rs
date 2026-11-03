use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use rename_files::{
    compare_entries, plan_entry_with_metadata, CleanupOptions, EntryMatch, RenameOp, RenamePlan,
    RenameTarget,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process;

/// Lossy display name for a path, escaped by `terminal_safe`, with a stable
/// fallback when `file_name()` is `None` (e.g. a path ending in `..`). Avoids
/// the panicky `unwrap()` on `file_name()` in the rename / conflict reporting
/// code paths.
fn display_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .map_or(Cow::Borrowed("?"), |n| terminal_safe(n.to_string_lossy()))
}

/// Whether a character from a name must be escaped before it is printed: C0
/// and C1 controls (newline, ESC, BEL, the 8-bit CSI…), the line and paragraph
/// separators, and the bidirectional formatting characters.
fn needs_escape(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

/// `text` with every character of `needs_escape` replaced by its Rust escape
/// (`\n`, `\u{1b}`, `\u{202e}`). Printed raw, a file name could drive the
/// terminal (clear the screen, set the title), add fake lines to the report,
/// or display the rest of the line reversed.
fn terminal_safe(text: Cow<'_, str>) -> Cow<'_, str> {
    if !text.chars().any(needs_escape) {
        return text;
    }
    let mut escaped = String::with_capacity(text.len() + 16);
    for c in text.chars() {
        if needs_escape(c) {
            escaped.extend(c.escape_debug());
        } else {
            escaped.push(c);
        }
    }
    Cow::Owned(escaped)
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
    #[arg(short = 'f', long = "files-only", conflicts_with = "dirs_only")]
    files_only: bool,

    /// Rename directories only
    #[arg(short = 'd', long = "dirs-only", conflicts_with = "files_only")]
    dirs_only: bool,

    /// Repair mojibake (UTF-8 wrongly decoded as Latin-1/CP1252) before
    /// renaming, e.g. "CafÃ©.txt" is treated as "Café.txt"
    #[arg(short = 'U', long = "fix-unicode")]
    fix_unicode: bool,

    // The help text is an explicit attribute rather than a doc comment: the
    // "<b>" example would trip rustdoc's invalid_html_tags lint.
    #[arg(
        short = 'H',
        long = "fix-html",
        help = "Strip HTML tags and decode HTML entities before renaming, \
                e.g. \"<b>Tom &amp; Jerry.txt\" is treated as \"Tom & Jerry.txt\""
    )]
    fix_html: bool,

    /// Apply every cleanup fix: -U and -H
    #[arg(short = 'A', long = "fix-all")]
    fix_all: bool,

    /// Delete a source that is a byte-for-byte duplicate of an existing
    /// destination, instead of reporting the name clash as an error
    #[arg(short = 'D', long = "delete-duplicates")]
    delete_duplicates: bool,

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
    /// `-D`: delete a source that strictly duplicates its destination.
    delete_duplicates: bool,
    target: RenameTarget,
    cleanup: CleanupOptions,
}

/// The first argument that clap reads as an option although an entry of that
/// name exists, which is what a glob such as `*` produces in a directory
/// holding a file called `-D`: the shell hands the name over and clap cannot
/// tell it from the option. A command line that contains `--` is trusted, as
/// the user has separated the options from the file names.
///
/// `args` excludes the program name. No option takes a value, so every
/// argument starting with `-` (other than `-` itself) is read as an option.
fn option_named_like_entry(args: &[OsString]) -> Option<&OsStr> {
    if args.iter().any(|arg| arg == "--") {
        return None;
    }
    args.iter().map(OsString::as_os_str).find(|arg| {
        let bytes = arg.as_encoded_bytes();
        bytes.len() > 1 && bytes[0] == b'-' && Path::new(arg).symlink_metadata().is_ok()
    })
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

/// Collect the cleanup fixes from the parsed flags; `-A` enables them all.
fn cleanup_from(cli: &Cli) -> CleanupOptions {
    CleanupOptions {
        fix_unicode: cli.fix_unicode || cli.fix_all,
        fix_html: cli.fix_html || cli.fix_all,
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
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const GREY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Wrap `text` in an ANSI colour escape, but only when `enabled` (the target
/// stream is a terminal). Piped / captured output stays plain.
fn paint(enabled: bool, code: &'static str, text: &'static str) -> Cow<'static, str> {
    if enabled {
        Cow::Owned(format!("{code}{text}{RESET}"))
    } else {
        Cow::Borrowed(text)
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

/// "duplicate" for 0 or 1, "duplicates" for more than one.
fn plural_duplicates(n: usize) -> &'static str {
    if n > 1 {
        "duplicates"
    } else {
        "duplicate"
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

/// Grey `parent:` prefix locating the directory the entry lives in. The current
/// directory is shown as `.`. Coloured only when `tty` is set.
fn location(path: &Path, tty: bool) -> String {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => terminal_safe(p.to_string_lossy()),
        _ => Cow::Borrowed("."),
    };
    if tty {
        format!("{GREY}{dir}:{RESET}")
    } else {
        format!("{dir}:")
    }
}

/// Prints the per-entry lines and the final report, honouring the verbosity
/// level and disabling colours when the streams are not terminals.
struct Reporter {
    verbosity: Verbosity,
    stdout_tty: bool,
    stderr_tty: bool,
    renamed: usize,
    duplicates: usize,
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
            duplicates: 0,
            errors: 0,
            skipped: 0,
        }
    }

    /// Report a successful (or, in dry-run, planned) rename:
    /// `[R] dir: source -> dest`.
    fn renamed(&mut self, from: &Path, to: &str) {
        self.renamed += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stdout_tty, MAGENTA, "R");
        let loc = location(from, self.stdout_tty);
        let name = display_name(from);
        let arrow = paint(self.stdout_tty, MAGENTA, "->");
        println!("[{mark}] {loc} {name} {arrow} {to}");
    }

    /// Report a source dropped because it duplicates its destination:
    /// `[W] dir: source -> message`. A warning, not an error: the wanted name
    /// is already there, holding the very same bytes.
    fn duplicate(&mut self, source: &Path, message: &str) {
        self.duplicates += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stderr_tty, YELLOW, "W");
        let loc = location(source, self.stderr_tty);
        let name = display_name(source);
        let arrow = paint(self.stderr_tty, YELLOW, "->");
        eprintln!("[{mark}] {loc} {name} {arrow} {message}");
    }

    /// Report a per-entry error: `[E] dir: source -> message`.
    fn error(&mut self, source: &Path, message: &str) {
        self.errors += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stderr_tty, RED, "E");
        let loc = location(source, self.stderr_tty);
        let name = display_name(source);
        let arrow = paint(self.stderr_tty, RED, "->");
        eprintln!("[{mark}] {loc} {name} {arrow} {message}");
    }

    /// Report an entry that matched but needed no rename: `[X] dir: source`.
    /// Only shown in verbose mode.
    fn skipped(&mut self, source: &Path) {
        self.skipped += 1;
        if self.verbosity != Verbosity::Verbose {
            return;
        }
        let mark = paint(self.stdout_tty, GREEN, "X");
        let loc = location(source, self.stdout_tty);
        let name = display_name(source);
        println!("[{mark}] {loc} {name}");
    }

    /// Print the final one-line summary. Silent in quiet mode.
    fn report(&self) {
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let matched = self.renamed + self.duplicates + self.errors + self.skipped;
        let matched_word = plural_entries(matched);
        let renamed_word = plural_entries(self.renamed);
        let errors_word = plural_errors(self.errors);
        // The duplicate segment only shows up when there is something to say,
        // keeping the usual summary as short as it has always been.
        let duplicates = if self.duplicates > 0 {
            format!(
                ", {} {} removed",
                self.duplicates,
                plural_duplicates(self.duplicates)
            )
        } else {
            String::new()
        };
        println!(
            "{matched} {matched_word} matched, {} {renamed_word} renamed{duplicates}, {} {errors_word}.",
            self.renamed, self.errors
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Conflict detection
// ─────────────────────────────────────────────────────────────────────────────

/// Return ops that are safe to apply (no destination conflicts).
/// Conflicting ops are reported and dropped.
///
/// An already existing destination is an error, except under `-D` when it is a
/// strict duplicate of the source (see `resolve_existing`), in which case the
/// source is deleted instead.
fn filter_conflicts(ops: Vec<RenameOp>, config: &Config, reporter: &mut Reporter) -> Vec<RenameOp> {
    let mut dest_count: HashMap<&Path, usize> = HashMap::with_capacity(ops.len());
    for op in &ops {
        *dest_count.entry(op.to.as_path()).or_insert(0) += 1;
    }

    let keep: Vec<bool> = ops
        .iter()
        .map(|op| {
            if dest_count[op.to.as_path()] > 1 {
                reporter.error(&op.from, "Multiple entries would produce this name");
                false
            } else if op.to.exists() {
                resolve_existing(op, config, reporter);
                false
            } else {
                true
            }
        })
        .collect();

    ops.into_iter()
        .zip(keep)
        .filter_map(|(op, keep)| keep.then_some(op))
        .collect()
}

/// Decide what to do with an op whose destination is already taken. Nothing is
/// ever overwritten, and nothing is ever deleted without `-D`.
///
/// When both sides are regular files with byte-for-byte identical content and
/// the same access rights (`same_access`), the rename would only produce a
/// copy of what is already there: the source is a redundant duplicate,
/// deleted under `-D` (or, with `--dry-run`, merely announced) and otherwise
/// reported as an error naming the flag that would clear it. Everything else
/// — differing content or access rights, directories, two names for the same
/// entry — stays a plain error and touches nothing.
fn resolve_existing(op: &RenameOp, config: &Config, reporter: &mut Reporter) {
    match compare_entries(&op.from, &op.to) {
        Ok(EntryMatch::Identical) => match same_access(&op.from, &op.to) {
            Ok(true) => delete_duplicate(op, config, reporter),
            Ok(false) => reporter.error(
                &op.from,
                "File name already exists (identical content, other owner or permissions)",
            ),
            Err(e) => reporter.error(&op.from, &format!("File name already exists ({e})")),
        },
        Ok(EntryMatch::Different) => {
            reporter.error(&op.from, "File name already exists (2 different files)");
        }
        Ok(EntryMatch::SameEntry | EntryMatch::NotComparable) => {
            reporter.error(&op.from, "File name already exists");
        }
        Err(e) => reporter.error(&op.from, &format!("File name already exists ({e})")),
    }
}

/// Handle a source that strictly duplicates its destination: delete it under
/// `-D` (only announce it with `--dry-run`), report the clash otherwise.
fn delete_duplicate(op: &RenameOp, config: &Config, reporter: &mut Reporter) {
    if !config.delete_duplicates {
        reporter.error(
            &op.from,
            "File name already exists (identical duplicate, use -D to delete the source)",
        );
    } else if config.dry_run {
        reporter.duplicate(&op.from, "Identical duplicate, source would be deleted");
    } else if let Err(e) = std::fs::remove_file(&op.from) {
        reporter.error(&op.from, &e.to_string());
    } else {
        reporter.duplicate(&op.from, "Identical duplicate, source deleted");
    }
}

/// Whether two regular files grant the same access: same owner, group and
/// permission bits on Unix, same read-only attribute elsewhere. The copy kept
/// by `-D` stands in for the deleted source, so it must not give anyone more
/// or less access: in a shared directory, another user could otherwise plant
/// an identical copy of their own and end up holding the only one left.
#[cfg(unix)]
fn same_access(a: &Path, b: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let (a, b) = (std::fs::symlink_metadata(a)?, std::fs::symlink_metadata(b)?);
    Ok(a.uid() == b.uid() && a.gid() == b.gid() && a.mode() & 0o7777 == b.mode() & 0o7777)
}

#[cfg(not(unix))]
fn same_access(a: &Path, b: &Path) -> std::io::Result<bool> {
    let (a, b) = (std::fs::symlink_metadata(a)?, std::fs::symlink_metadata(b)?);
    Ok(a.permissions().readonly() == b.permissions().readonly())
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
        let to_name = display_name(&op.to);

        if dry_run {
            reporter.renamed(&op.from, &to_name);
        } else {
            match rename_no_clobber(&op.from, &op.to) {
                Ok(()) => reporter.renamed(&op.from, &to_name),
                Err(e) => reporter.error(&op.from, &e.to_string()),
            }
        }
    }
}

/// Whether the entry's own name is valid UTF-8, the precondition every
/// transformation in the library works on.
fn has_utf8_name(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n.to_str().is_some())
}

/// Rename a list of explicitly-named entries (the `rename`-like mode).
///
/// Each path is renamed **itself** (not its contents). Missing paths are
/// reported and counted as errors but do not abort the batch. Destination
/// conflicts across the whole batch are detected together via `filter_conflicts`.
fn process_targets(paths: &[PathBuf], config: &Config, reporter: &mut Reporter) {
    let mut ops = Vec::new();
    for path in paths {
        // A single `stat` per argument: it detects a missing path and feeds
        // both the planning and the `Excluded` diagnosis below.
        let Ok(meta) = path.metadata() else {
            reporter.error(path, "No such file or directory");
            continue;
        };
        match plan_entry_with_metadata(path, &meta, config.target, config.cleanup) {
            RenamePlan::Rename(op) => ops.push(op),
            RenamePlan::AlreadyClean => reporter.skipped(path),
            // `Excluded` covers two very different cases: an entry the type
            // filter deliberately left out (stay silent), and a name that is
            // not valid UTF-8 (report it — the user named this path
            // explicitly and would otherwise get no feedback at all).
            RenamePlan::Excluded => {
                if config.target.matches(&meta) && !has_utf8_name(path) {
                    reporter.error(path, "Name is not valid UTF-8");
                }
            }
        }
    }

    // Rename the deepest entries first: a descendant path always has more
    // components than its ancestor, so sorting by descending depth guarantees
    // that files and leaf directories are renamed before the directories that
    // contain them. Otherwise renaming a parent first would invalidate the
    // stored child paths and make their renames fail with ENOENT.
    let mut ops = filter_conflicts(ops, config, reporter);
    ops.sort_by_cached_key(|op| std::cmp::Reverse(op.from.components().count()));
    apply_ops(&ops, config.dry_run, reporter);
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    // `Cli::parse_from` exits the process on parse errors and on `--help` /
    // `--version`, so anything reaching the next check already has a
    // syntactically valid command line.
    let args: Vec<OsString> = std::env::args_os().collect();
    let cli = Cli::parse_from(&args);

    // Refuse to guess before acting, and before `-q` could silence anything:
    // the "option" may be a file name the shell expanded from a glob.
    if let Some(arg) = option_named_like_entry(args.get(1..).unwrap_or_default()) {
        Cli::command()
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "{arg:?} is read as an option but also names an existing file or \
                     directory\n\n  tip: put '--' before the file names, as in \
                     'rename-simple -- *', or glob './*' instead of '*'"
                ),
            )
            .exit();
    }

    // Without explicit paths there is nothing to operate on: show help like -h.
    if cli.paths.is_empty() {
        Cli::command().print_help().unwrap_or(());
        process::exit(0);
    }

    let config = Config {
        dry_run: cli.dry_run,
        delete_duplicates: cli.delete_duplicates,
        target: target_from(&cli),
        cleanup: cleanup_from(&cli),
    };

    let mut reporter = Reporter::new(verbosity_from(&cli));
    process_targets(&cli.paths, &config, &mut reporter);
    reporter.report();
}
