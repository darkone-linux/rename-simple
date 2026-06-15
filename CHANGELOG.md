# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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