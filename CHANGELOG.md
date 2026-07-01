# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-06-30

### Added
- **Rename-like mode**: paths can now be passed explicitly on the command line
  and each argument is renamed **itself** (not its contents), like the
  traditional `rename`(1) command. Globbing is left to the shell, so
  `rename-simple *.jpg` or `rename-simple dir/**/*.pdf` (with `globstar`) work
  as expected. New public `plan_rename` function backs this mode.
- **Structured, colourised output** with three verbosity levels:
  - `-q`/`--quiet`: print nothing at all (errors included);
  - default: one line per renamed (`[R] source -> dest`) or errored
    (`[E] source -> message`) entry, followed by a summary;
  - `-v`/`--verbose`: also list untouched entries (`[X] source`).

  The `R`/`E`/`X` marks and the `->` arrow are colourised (magenta / red /
  green) only when the stream is a terminal, so piped output stays plain. The
  summary reads `N entries matched, N entries renamed, N errors.` with correct
  singular/plural forms.
- New public `plan_entry` function and `RenamePlan` enum, distinguishing an
  already-clean entry (reported as `[X]`) from one excluded by the type filter
  or an invalid-UTF-8 name.

### Fixed
- **Parent renamed before its contents**: when a directory and entries inside
  it were passed together (e.g. `rename-simple **/*`), renaming the parent
  first invalidated the child paths and made their renames fail with ENOENT.
  Renames are now applied deepest-first, so files and leaf directories are
  handled before the directories that contain them.

### Removed
- **BREAKING — directory-scan mode**: `rename-simple` no longer reads the
  current directory when invoked without paths; it operates only on the
  arguments it is given. With no argument, it prints its help.
- **BREAKING — `-a`/`--all` flag**: redundant now that both files and
  directories are renamed by default. Use `rename-simple *` instead.
- **BREAKING — `-r`/`--recursive` flag**: recursion is dropped. Use the shell's
  recursive globbing (`rename-simple **/*.pdf`) to reach nested entries.
- Public `compute_renames` function (the directory scanner) removed from the
  library API.

### Changed
- `-f` / `-d` now act purely as a type filter on the explicit arguments.
- **Output redesign**: the default mode is no longer silent — it prints the
  `[R]`/`[E]` lines plus a summary. Use `-q` to restore fully silent behaviour.
  Conflicts and errors are reported as `[E]` lines (`-q` suppresses them too).
- README, man page and `AGENTS.md` updated for the rename-like model; the
  shell-alias tip is now `alias rsa='rename-simple *'`.
- Tests reworked around explicit paths; `tests/recursive_tests.rs` removed.

## [0.3.1] - 2026-06-15

### Fixed
- Replaced a `map(...).unwrap_or(false)` on a `Result` in `collect_subdirs`
  with `is_ok_and(...)`: newer clippy flags the former under
  `clippy::map_unwrap_or`, which broke CI even though the code was correct.
- `just release` now sends a `User-Agent` header when querying the crates.io
  API to detect an already-published version. crates.io answers `403` to
  requests without one, so the guard always fell through to `cargo publish`
  and errored on an existing version instead of skipping cleanly. The release
  recipe is idempotent again.

### Changed
- `shell.nix` now pins `nixpkgs` to an exact revision instead of the ambient
  `<nixpkgs>` channel, so local dev shells and CI resolve the same toolchain
  (cargo, clippy, rustfmt). This prevents clippy-version drift from surfacing
  lints in CI that `just test` could not see locally.

## [0.3.0] - 2026-06-15

### Added
- Public `transform_dirname` function: directories have no extension, so a dot
  in their name is treated as a plain separator and the whole name goes through
  the slug pipeline (e.g. `My Project.v2` → `my-project-v2`, where the old
  extension-aware path produced `my-project.v2`).

### Fixed
- **Recursion into the wrong sibling on a name conflict**: when a directory
  rename is blocked because a sibling already holds the target name, `-r` now
  descends into the blocked directory under its original (unchanged) name
  instead of the pre-existing sibling. Recursion follows the renames actually
  applied to disk rather than recomputing the destination and probing the
  filesystem. Closes the documented limitation in the README.

### Changed
- Directory names are now slugified via `transform_dirname` instead of
  `transform_filename`, so a dot in a directory name no longer survives as a
  spurious `.ext` suffix.

### Dependencies
- Updated `rustix` 0.38 → 1; refreshed compatible lockfile entries. The Linux
  dependency tree shrank from 95 to 83 crates (no more `windows-sys`,
  deduplicated with `tempfile`).

## [0.2.4] - 2026-05-19

### Security
- **Recursive symlink escape (C2)**: `-r` no longer follows symlinks that
  point at directories. `collect_subdirs` now uses `symlink_metadata`, so
  a symlink inside the target tree cannot redirect recursion to files
  outside that tree or trigger an unbounded loop.
- **TOCTOU on rename (C1)**: a new `rename_no_clobber` helper replaces
  `fs::rename`. On Linux it uses `renameat2(RENAME_NOREPLACE)` via
  `rustix`, closing the race window between the `op.to.exists()` pre-check
  and the syscall itself. On other Unix and Windows, behaviour is
  `try_exists` + `fs::rename` (Windows `rename` is already non-clobbering).

### Added
- Invalid-UTF-8 entries now produce a stderr warning in `-v` mode instead
  of being silently dropped from the report.
- 19 new tests covering the audit surface:
  - 10 in `tests/transform_tests.rs` — RTL scripts, ZWJ emoji, variation
    selector, NUL byte, RTL override, path-traversal segments, `unnamed`
    collision, NFD/NFC equivalence.
  - 3 in `tests/cli_tests.rs` — `unnamed.<ext>` collisions and
    dry-run-preserves-data.
  - 3 in `tests/recursive_tests.rs` — 15-level deep nesting, descent into
    renamed directory, independent sibling subdirectories.
  - 3 in `tests/unix_tests.rs` — directory-symlink not followed, symlink
    loop terminates, read-only parent does not panic, invalid-UTF-8
    verbose warning.

### Changed
- Replaced four `file_name().unwrap()` calls in the rename / conflict
  reporting paths with a `display_name` helper that falls back to `"?"`
  when the file name is absent.

### Dependencies
- Added `rustix 0.38` (Linux target only, `fs` feature) for the
  `renameat2(RENAME_NOREPLACE)` syscall wrapper.

## [0.2.3] - 2026-05-15

### Added
- Extension validation: extensions containing non-ASCII or non-alphanumeric
  characters are now absorbed into the stem and transliterated
- Extension length limit: extensions longer than 10 ASCII alphanumeric
  characters are treated as part of the stem
- 9 new tests covering extension validity edge cases

### Changed
- `split_extension` now requires extensions to be purely ASCII alphanumeric
  and ≤10 characters long; invalid extensions re-enter the stem pipeline

## [0.2.2] - 2026-05-08

### Added
- `-a`, `--all` flag to rename both files and directories (replaces the former implicit default)
- `--version` / `-V` flag (exposed via clap)
- Shell alias tip in README: `alias rsa='rename-simple -a'`
- `tests/unix_tests.rs`: new integration tests covering symlinks and invalid UTF-8 filenames
- Justfile: `just bump [patch|minor|major]` and `just release` to automate version bumps and publishing

### Changed
- Running `rename-simple` without a target-mode flag (`-f`, `-d`, or `-a`) now prints help and exits cleanly instead of processing the current directory
- `-f`, `-d`, and `-a` are mutually exclusive; combining any two is rejected at parse time
- README examples updated to reflect the new explicit flags
- Build hardened: `clippy::pedantic` enforced, `unsafe_code = "deny"`, MSRV pinned to 1.70, release profile optimised (`lto`, `strip`, `panic = "abort"`)

## [0.2.1] - 2026-05-07

### Added
- `-v`, `--verbose` flag to display rename details and summary
- Man page (`man/rename-simple.1`)

### Changed
- Program is now silent by default (no stdout output) unless `-v` is used
- Errors and conflict warnings still go to stderr regardless of `-v`

## [0.2.0] - 2026-04-29

### Added
- `-r`, `--recursive` flag to process subdirectories recursively
- New test file `tests/recursive_tests.rs` with 9 tests for recursive functionality

### Changed
- Updated documentation (README.md, AGENTS.md) to include `-r` option
- Test structure improved with separate test files for CLI and recursive tests

## [0.1.0] - 2026-04-28

### Added
- Initial release
- Core filename transformation:
  - Transliteration of accented/extended Latin characters (é→e, ç→c, œ→oe, ß→ss, etc.)
  - Lowercase conversion
  - Space and special character replacement with `-`
  - `_` preservation and cleanup of `_-` / `-_` sequences
  - Leading/trailing separator stripping
- CLI flags:
  - `-f`, `--files`: Rename files only
  - `-d`, `--dirs`: Rename directories only
  - `-n`, `--dry-run`: Preview without making changes
  - `-h`, `--help`: Display help
- Compound extension preservation (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`)
- Hidden file/folder skipping (`.gitignore`, etc.)
- Conflict detection (multiple files renaming to same target)
- Test suite (`tests/transform_tests.rs`, `tests/cli_tests.rs`)

### Fixed
- Extension handling for known double extensions
- Various edge cases in character transliteration