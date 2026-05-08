# rename-simple — Agent Instructions

## Project Overview

- **Goal**: A small Rust CLI tool that renames files in a directory to clean, ASCII-safe slugs.
- **Language**: Rust (edition 2021, MSRV 1.70).
- **Build**: `cargo build --release` → `target/release/rename-simple`
- **Quality gate**: `just test` (runs `fmt-check`, `lint`, `unit`, `audit`, `doc`).
- **Development**: Use `nix-shell` to load `cargo`, `rustc`, `clippy`, `rustfmt`, `cargo-audit`, `just`, `gh` and `nixfmt` with pinned versions.

---

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

---

## Current Status

- All code, comments, and documentation in English
- Tests located in `tests/` directory
- Hidden files (starting with `.`) are skipped
- Compound extensions (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`) are preserved
- Recursive processing (`-r`) is implemented

---

## Code Conventions

- **All code, comments, docstrings must be in English**, except communication with user.
- **Formatting**: Code MUST be formatted using `rustfmt` before every commit.
- **Linter**: Code MUST be compliant with `clippy::pedantic` (configured via `[lints]` in `Cargo.toml`, no warnings allowed).
- **Unsafe code**: Forbidden — `unsafe_code = "deny"` is enforced at the crate level.
- **Tests location**: `tests/` directory.
- **Test naming**: `tests/test_*.rs` or `tests/*_tests.rs`.

---

## CLI Usage

```
rename-simple [OPTIONS] [DIR]

Arguments:
  DIR             Target directory (default: current directory)

Options:
  -f              Rename files only (default: files + directories)
  -d              Rename directories only
  -r, --recursive Process subdirectories recursively
  -v, --verbose   Show details of each rename and a summary
  -n, --dry-run   Show what would be renamed without touching any entry
  -h, --help      Print this help message
```

Exit status is `0` on success (including when nothing needs to be renamed) and `1`
on any error (invalid arguments, unreadable target directory, etc.).
By default the program produces no output on success: details only appear with `-v`.

---

## Error Handling

- **Conflict detection**: Multiple files that would rename to the same destination are skipped with a warning.
- **Existing destination**: Files that already exist at the target path are skipped with a warning.
- **Hidden files**: Files starting with `.` are ignored.
- **IO errors**: Reported per-file with error count in summary.

---

## Validation & Quality Gate

The Agent must execute and pass the following "Checklist" before proposing a
solution or finishing a task. **Use `just test` — it runs the whole gate in
the correct order.**

| # | Step                  | Justfile recipe | Underlying command                                              |
|---|-----------------------|-----------------|------------------------------------------------------------------|
| 1 | Format check          | `just fmt-check`| `cargo fmt --all --check`                                        |
| 2 | Linting check         | `just lint`     | `cargo clippy --all-targets --all-features -- -D warnings`       |
| 3 | Logic verification    | `just unit`     | `cargo test`                                                     |
| 4 | Security audit        | `just audit`    | `cargo audit`                                                    |
| 5 | Documentation check   | `just doc`      | `cargo doc --no-deps`                                            |

Auto-fix shortcut (formatter + clippy `--fix`): `just fix`.

---

## TDD Workflow

1. **Write failing test first** — In the `tests/` directory.
2. **Run tests** — `just unit` (validate red phase).
3. **Implement minimal code** — Make the test pass (green phase).
4. **Clean up** — `just fix` (clippy auto-fix + format).
5. **Refactor** — Improve code while keeping tests green.
6. **After 5 iterations**, stop and ask the user for direction.
7. **Final validation** — Run `just test` (full Validation & Quality Gate) and fix any issues.

---

## Language

- **Code, comments, docstrings, and documentation**: Always in English
- **Communication with user**: Respond in the user's language (French in this case)
