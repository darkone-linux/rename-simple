use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use rename_files::{
    compare_entries, plan_entry_with_metadata, CleanupOptions, EntryMatch, RenameOp, RenamePlan,
    RenameTarget,
};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{self, ExitCode};

/// Lossy display name for a path, escaped by `terminal_safe`. A path without
/// a name (`.`, `x/..`, `/`) shows its last component instead, and an empty
/// path `?`. Avoids the panicky `unwrap()` on `file_name()` in the rename /
/// conflict reporting code paths.
fn display_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .or_else(|| {
            path.components()
                .next_back()
                .map(std::path::Component::as_os_str)
        })
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

/// What the reporter knows about one standard stream.
struct Stream {
    /// Colours are only used on a terminal.
    tty: bool,
    /// Cleared by the first failed write, see `write_line`.
    open: bool,
}

impl Stream {
    fn new(tty: bool) -> Self {
        Self { tty, open: true }
    }

    /// Write one line to `handle`, this stream's handle, unless an earlier
    /// write failed. A failed write (the reader of a pipe is gone, as with
    /// `| head`, or the disk is full) silences the stream for the rest of the
    /// run instead of aborting it the way `println!` does: the renames are the
    /// job, the report only describes them.
    fn write_line(&mut self, mut handle: impl Write, line: fmt::Arguments<'_>) {
        if self.open && writeln!(handle, "{line}").is_err() {
            self.open = false;
        }
    }
}

/// Prints the per-entry lines and the final report, honouring the verbosity
/// level and disabling colours when the streams are not terminals.
struct Reporter {
    verbosity: Verbosity,
    stdout: Stream,
    stderr: Stream,
    renamed: usize,
    duplicates: usize,
    errors: usize,
    skipped: usize,
}

impl Reporter {
    fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            stdout: Stream::new(std::io::stdout().is_terminal()),
            stderr: Stream::new(std::io::stderr().is_terminal()),
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
        let mark = paint(self.stdout.tty, MAGENTA, "R");
        let loc = location(from, self.stdout.tty);
        let name = display_name(from);
        let arrow = paint(self.stdout.tty, MAGENTA, "->");
        self.stdout.write_line(
            std::io::stdout().lock(),
            format_args!("[{mark}] {loc} {name} {arrow} {to}"),
        );
    }

    /// Report a source dropped because it duplicates its destination:
    /// `[W] dir: source -> message`. A warning, not an error: the wanted name
    /// is already there, holding the very same bytes.
    fn duplicate(&mut self, source: &Path, message: &str) {
        self.duplicates += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stderr.tty, YELLOW, "W");
        let loc = location(source, self.stderr.tty);
        let name = display_name(source);
        let arrow = paint(self.stderr.tty, YELLOW, "->");
        self.stderr.write_line(
            std::io::stderr().lock(),
            format_args!("[{mark}] {loc} {name} {arrow} {message}"),
        );
    }

    /// Report a per-entry error: `[E] dir: source -> message`.
    fn error(&mut self, source: &Path, message: &str) {
        self.errors += 1;
        if self.verbosity == Verbosity::Quiet {
            return;
        }
        let mark = paint(self.stderr.tty, RED, "E");
        let loc = location(source, self.stderr.tty);
        let name = display_name(source);
        let arrow = paint(self.stderr.tty, RED, "->");
        self.stderr.write_line(
            std::io::stderr().lock(),
            format_args!("[{mark}] {loc} {name} {arrow} {message}"),
        );
    }

    /// Report an entry that matched but needed no rename: `[X] dir: source`.
    /// Only shown in verbose mode.
    fn skipped(&mut self, source: &Path) {
        self.skipped += 1;
        if self.verbosity != Verbosity::Verbose {
            return;
        }
        let mark = paint(self.stdout.tty, GREEN, "X");
        let loc = location(source, self.stdout.tty);
        let name = display_name(source);
        self.stdout.write_line(
            std::io::stdout().lock(),
            format_args!("[{mark}] {loc} {name}"),
        );
    }

    /// Print the final one-line summary. Silent in quiet mode.
    fn report(&mut self) {
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
        self.stdout.write_line(
            std::io::stdout().lock(),
            format_args!(
                "{matched} {matched_word} matched, {} {renamed_word} renamed{duplicates}, {} {errors_word}.",
                self.renamed, self.errors
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Conflict detection
// ─────────────────────────────────────────────────────────────────────────────

/// A rename that passed the conflict checks.
struct Rename {
    op: RenameOp,
    /// The new name already designates the entry itself: on a case-insensitive
    /// filesystem, `photo.jpg` is another spelling of `Photo.JPG`. Applied by
    /// `respell`.
    same_entry: bool,
}

/// The directory holding `path`: its parent, or `.` for a bare name.
fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Where an entry lives, however its path is spelled: the parent directory
/// canonicalized (symlinks, `.` and `..` resolved) joined with the entry's own
/// name, left as is since the entry itself may be a symlink. `Report A.pdf`,
/// `./Report A.pdf` and `sub/../Report A.pdf` share one key. `None` for a path
/// without a name (`..`, `/`).
fn entry_key(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?;
    let dir = parent_dir(path);
    let dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    Some(dir.join(name))
}

/// The names stored in each directory, listed once, on first use. A lookup
/// may fold case (`photo.jpg` finds `Photo.JPG`); a listing returns the names
/// as stored.
#[derive(Default)]
struct Listings(HashMap<PathBuf, HashSet<OsString>>);

impl Listings {
    /// Whether the directory holding `path` stores an entry spelled exactly
    /// like its last component.
    fn has_entry(&mut self, path: &Path) -> std::io::Result<bool> {
        let Some(name) = path.file_name() else {
            return Ok(true);
        };
        let dir = parent_dir(path);
        if !self.0.contains_key(dir) {
            let names = std::fs::read_dir(dir)?
                .map(|entry| entry.map(|e| e.file_name()))
                .collect::<std::io::Result<_>>()?;
            self.0.insert(dir.to_path_buf(), names);
        }
        Ok(self.0[dir].contains(name))
    }
}

/// Return the renames that are safe to apply (no destination conflicts).
/// Conflicting ops are reported and dropped.
///
/// An already existing destination is an error, except when it is the source
/// itself under another spelling, and under `-D` when it is a strict
/// duplicate of the source, in which case the source is deleted instead (see
/// `resolve_existing`).
fn filter_conflicts(ops: Vec<RenameOp>, config: &Config, reporter: &mut Reporter) -> Vec<Rename> {
    // Destinations are told apart by location, not by spelling: `note-b.txt`
    // and `./note-b.txt` are one name.
    let dest_keys: Vec<PathBuf> = ops
        .iter()
        .map(|op| entry_key(&op.to).unwrap_or_else(|| op.to.clone()))
        .collect();
    let mut dest_count: HashMap<&Path, usize> = HashMap::with_capacity(ops.len());
    for key in &dest_keys {
        *dest_count.entry(key.as_path()).or_insert(0) += 1;
    }

    // `Some(same_entry)` keeps the op, `None` drops it.
    let mut listings = Listings::default();
    let keep: Vec<Option<bool>> = ops
        .iter()
        .zip(&dest_keys)
        .map(|(op, key)| {
            if dest_count[key.as_path()] > 1 {
                reporter.error(&op.from, "Multiple entries would produce this name");
                None
            } else if taken(&op.to) {
                resolve_existing(op, config, &mut listings, reporter).then_some(true)
            } else {
                Some(false)
            }
        })
        .collect();

    ops.into_iter()
        .zip(keep)
        .filter_map(|(op, keep)| keep.map(|same_entry| Rename { op, same_entry }))
        .collect()
}

/// Whether an entry already holds the name `path`, whatever it is. A symlink
/// pointing nowhere takes its name all the same, which `Path::exists` misses
/// since it follows the link.
fn taken(path: &Path) -> bool {
    path.symlink_metadata().is_ok()
}

/// Decide what to do with an op whose destination is already taken, and
/// return whether the rename must still happen. Nothing is ever overwritten,
/// and nothing is ever deleted without `-D`.
///
/// The rename still happens when the destination is the source itself under
/// another spelling: a case-insensitive filesystem resolves `photo.jpg` to
/// `Photo.JPG`, yet stores no entry spelled `photo.jpg`. When both sides are
/// regular files with byte-for-byte identical content and the same access
/// rights (`same_access`), the rename would only produce a copy of what is
/// already there: the source is a redundant duplicate, deleted under `-D` (or,
/// with `--dry-run`, merely announced) and otherwise reported as an error
/// naming the flag that would clear it. Everything else — differing content or
/// access rights, directories, two entries of one file (hard links) — stays a
/// plain error and touches nothing.
fn resolve_existing(
    op: &RenameOp,
    config: &Config,
    listings: &mut Listings,
    reporter: &mut Reporter,
) -> bool {
    match compare_entries(&op.from, &op.to) {
        Ok(EntryMatch::SameEntry) => match listings.has_entry(&op.to) {
            Ok(false) => return true,
            Ok(true) => reporter.error(&op.from, "File name already exists"),
            Err(e) => reporter.error(&op.from, &format!("File name already exists ({e})")),
        },
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
        Ok(EntryMatch::NotComparable) => {
            reporter.error(&op.from, "File name already exists");
        }
        Err(e) => reporter.error(&op.from, &format!("File name already exists ({e})")),
    }
    false
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
/// Closes the TOCTOU window between the `taken` pre-check in
/// `filter_conflicts` and the actual rename:
///
/// - **Linux**: `renameat2(AT_FDCWD, from, AT_FDCWD, to, RENAME_NOREPLACE)`
///   via `rustix` — atomic at the syscall level. A filesystem that does not
///   support the flag (NFS, FUSE without `rename2`: sshfs, ntfs-3g) or a
///   kernel without `renameat2` rejects the call itself, with `EINVAL`,
///   `ENOSYS` or `EOPNOTSUPP`: `check_then_rename` takes over, as in `mv -n`.
/// - **Other Unix / Windows**: `check_then_rename`.
fn rename_no_clobber(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{renameat_with, RenameFlags, CWD};
        use rustix::io::Errno;
        match renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE) {
            Err(Errno::INVAL | Errno::NOSYS | Errno::OPNOTSUPP) => check_then_rename(from, to),
            result => result.map_err(std::io::Error::from),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        check_then_rename(from, to)
    }
}

/// Rename `from` to `to` unless an entry already holds the name `to`, checked
/// as in `taken`. Two calls, not one: `std::fs::rename` replaces an existing
/// destination, so an entry created between the check and the rename would
/// be overwritten.
fn check_then_rename(from: &Path, to: &Path) -> std::io::Result<()> {
    match to.symlink_metadata() {
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "destination exists",
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => std::fs::rename(from, to),
        Err(e) => Err(e),
    }
}

/// Give `from` the name `to` when `to` is another spelling of `from` itself
/// (`Rename::same_entry`). rename(2) does nothing for two names of one file
/// and `RENAME_NOREPLACE` sees a clash, so the entry goes through a temporary
/// name in the same directory. If the second step fails, the entry gets its
/// original name back.
fn respell(from: &Path, to: &Path) -> std::io::Result<()> {
    let temp_name = format!(".rename-simple-{}", process::id());
    let temp = from.with_file_name(&temp_name);
    rename_no_clobber(from, &temp)?;
    let Err(e) = rename_no_clobber(&temp, to) else {
        return Ok(());
    };
    match rename_no_clobber(&temp, from) {
        Ok(()) => Err(e),
        Err(_) => Err(std::io::Error::new(
            e.kind(),
            format!("{e}, the entry is left as {temp_name}"),
        )),
    }
}

/// Apply the renames, reporting each result.
fn apply_ops(renames: &[Rename], dry_run: bool, reporter: &mut Reporter) {
    for Rename { op, same_entry } in renames {
        let to_name = display_name(&op.to);

        if dry_run {
            reporter.renamed(&op.from, &to_name);
            continue;
        }
        let result = if *same_entry {
            respell(&op.from, &op.to)
        } else {
            rename_no_clobber(&op.from, &op.to)
        };
        match result {
            Ok(()) => reporter.renamed(&op.from, &to_name),
            Err(e) => reporter.error(&op.from, &e.to_string()),
        }
    }
}

/// Whether the entry's own name is valid UTF-8, the precondition every
/// transformation in the library works on.
fn has_utf8_name(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n.to_str().is_some())
}

/// The error to report for an argument whose metadata cannot be read.
/// `metadata` follows symlinks, so a link to nothing fails with "not found"
/// although the link itself exists: say so, as "No such file" would send the
/// user looking for a typo. Other failures (permission denied, symlink loop…)
/// are reported as they are.
fn unreadable_entry(path: &Path, e: &std::io::Error) -> String {
    if e.kind() != std::io::ErrorKind::NotFound {
        e.to_string()
    } else if path.symlink_metadata().is_ok() {
        "Broken symbolic link".to_owned()
    } else {
        "No such file or directory".to_owned()
    }
}

/// `path` without its trailing `.` components: `My Dir/.` names the same
/// directory as `My Dir/`, but rename(2) refuses a path ending in `.` (EBUSY).
/// `Path::components` hides a trailing `.`, so the text is checked instead.
/// The trailing separator is kept, `My Dir/.` becoming `My Dir/`: like the
/// original, the result designates a directory.
fn without_trailing_dot(path: &Path) -> Cow<'_, Path> {
    let text = path.as_os_str().as_encoded_bytes();
    let is_separator = |b: &u8| std::path::is_separator(char::from(*b));
    let end = text
        .iter()
        .rposition(|b| !is_separator(b))
        .map_or(0, |i| i + 1);
    let text = &text[..end];
    if text.len() < 2 || !text.ends_with(b".") || !is_separator(&text[text.len() - 2]) {
        return Cow::Borrowed(path);
    }
    let mut dir: PathBuf = path.components().collect();
    dir.push("");
    Cow::Owned(dir)
}

/// Rename a list of explicitly-named entries (the `rename`-like mode).
///
/// Each path is renamed **itself** (not its contents). Missing paths are
/// reported and counted as errors but do not abort the batch. Destination
/// conflicts across the whole batch are detected together via `filter_conflicts`.
fn process_targets(paths: &[PathBuf], config: &Config, reporter: &mut Reporter) {
    let mut ops = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        let path = &*without_trailing_dot(path);
        // A single `stat` per argument: it detects a missing path and feeds
        // both the planning and the `Excluded` diagnosis below.
        let meta = match path.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                reporter.error(path, &unreadable_entry(path, &e));
                continue;
            }
        };
        // `.`, `x/..` and `/` refer to a directory without naming it.
        if path.file_name().is_none() {
            reporter.error(path, "Cannot rename ., .. or /");
            continue;
        }
        // Overlapping globs, or `x` next to `./x`: an entry named several
        // times is handled once.
        if entry_key(path).is_some_and(|key| !seen.insert(key)) {
            continue;
        }
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

    // Rename the deepest entries first: a descendant always lies deeper than
    // its ancestor, so sorting by descending depth guarantees that files and
    // leaf directories are renamed before the directories that contain them.
    // Otherwise renaming a parent first would invalidate the stored child
    // paths and make their renames fail with ENOENT. The depth is the one of
    // the entry's location (`entry_key`), not of the path as typed: an
    // absolute parent has more components than its relative child.
    let mut renames = filter_conflicts(ops, config, reporter);
    renames.sort_by_cached_key(|r| {
        let location = entry_key(&r.op.from).unwrap_or_else(|| r.op.from.clone());
        std::cmp::Reverse(location.components().count())
    });
    apply_ops(&renames, config.dry_run, reporter);
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

/// Exit status: `0` when every entry was handled, `1` when at least one was
/// reported as an error (`[E]`), `2` for an invalid command line (clap's own
/// usage errors, and an option that names an existing entry).
fn main() -> ExitCode {
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
        return ExitCode::SUCCESS;
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
    // As with `mv` or `cp`: one failed entry fails the command, in a dry-run
    // too (it predicts the real run) and under `-q` (the status is all that
    // is left then). The other entries have been processed all the same.
    if reporter.errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
