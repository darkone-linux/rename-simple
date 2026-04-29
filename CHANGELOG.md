# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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