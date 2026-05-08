# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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