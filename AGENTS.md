# rename-simple — Agent Instructions

Telegraph style. Root rules only. Read scoped `AGENTS.md` before subtree work.

## Project Overview

- **Goal**: Rename files to clean, ASCII-safe slugs.
- **Language**: Rust (edition 2021, MSRV 1.70).
- **Build**: `cargo build --release` → `target/release/rename-simple`
- **Quality gate**: `just test` (fmt-check, lint, unit, audit, doc).
- **Dev shell**: `nix-shell` — cargo, rustc, clippy, rustfmt, cargo-audit, just, gh, nixfmt (pinned).

## Project Structure

```
src/
├── lib.rs      # Core transformation logic
│               #   public API: transliterate_char, transform_stem,
│               #               transform_filename, compute_renames,
│               #               RenameOp, RenameTarget
└── main.rs     # CLI (argument parsing, conflict detection, recursion driver)
tests/
├── transform_tests.rs  # Pure unit tests for the transformation pipeline
├── cli_tests.rs        # End-to-end CLI integration tests
└── recursive_tests.rs  # Recursive (`-r`) integration tests
man/
└── rename-simple.1     # Man page (kept in sync with the CLI manually)
```

## Current Status

- All code, comments, and documentation in English
- Tests located in `tests/` directory
- Hidden files (starting with `.`) are skipped
- Compound extensions (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`) are preserved
- Recursive processing (`-r`) is implemented

## Code Conventions

- **Formatting**: `rustfmt` before every commit.
- **Linter**: `clippy::pedantic` (Cargo.toml `[lints]`), zero warnings allowed.
- **Unsafe code**: Forbidden — `unsafe_code = "deny"` at crate level.
- **Tests**: `tests/` directory.
- **Test naming**: `tests/test_*.rs` or `tests/*_tests.rs`.

## CLI Usage

```
Usage: rename-simple [OPTIONS] [DIR]

Arguments:
  [DIR]  Target directory (default: current directory)

Options:
  -f               Rename files only
  -d               Rename directories only
  -a, --all        Rename both files and directories
  -r, --recursive  Process subdirectories recursively
  -v, --verbose    Show details of what is being renamed
  -n, --dry-run    Show what would be renamed without touching any entry
  -h, --help       Print help
  -V, --version    Print version
```

Exit status: `0` on success (including no-op), `1` on error.
Default: no output; details only with `-v`.

## Error Handling

- **Conflict detection**: Multiple files that would rename to the same destination are skipped with a warning.
- **Existing destination**: Files that already exist at the target path are skipped with a warning.
- **Hidden files**: Files starting with `.` are ignored.
- **IO errors**: Reported per-file with error count in summary.

## Validation & Quality Gate

The Agent must execute and pass the following "Checklist" before proposing a
solution or finishing a task. **Use `just test` — it runs the whole gate in
the correct order.**

| # | Step                  | Justfile recipe | Underlying command                                               |
|---|-----------------------|-----------------|------------------------------------------------------------------|
| 1 | Format check          | `just fmt-check`| `cargo fmt --all --check`                                        |
| 2 | Linting check         | `just lint`     | `cargo clippy --all-targets --all-features -- -D warnings`       |
| 3 | Logic verification    | `just unit`     | `cargo test`                                                     |
| 4 | Security audit        | `just audit`    | `cargo audit`                                                    |
| 5 | Documentation check   | `just doc`      | `cargo doc --no-deps`                                            |

Auto-fix shortcut (formatter + clippy `--fix`): `just fix`.

## TDD Workflow

1. **Write failing test first** — In `tests/`.
2. **Run tests** — `just unit` (red phase).
3. **Implement minimal code** — Make tests pass (green phase).
4. **Clean up** — `just fix` (clippy auto-fix + format).
5. **Refactor** — Keep tests green.
6. **After 5 iterations**, stop and ask user.
7. **Final validation** — `just test`; fix any issues.

## Language

- **All code, comments, docstrings, documentation, man page**: English
- **User communication**: Respond in user's language (French here)

## Git

- **Read operations**: All permitted (status, diff, log, etc.).
- **Staging**: All permitted (add, restore, reset, etc.).
- **Commit**: On explicit request only. Message: 1 line, 50 chars max.
- **Push**: Never allowed.
- Never commit unless `just test` is green.
