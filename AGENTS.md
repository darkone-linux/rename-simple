# rename-simple — Agent Instructions

## Project Overview

- **Goal**: A small Rust CLI tool that renames files in a directory to clean, ASCII-safe slugs.
- **Language**: Rust
- **Build**: `cargo build --release` → `target/release/rename-simple`
- **Test**: `cargo test`
- **Development**: Use `nix-shell` to load cargo, rust and project dependencies.

---

## Project Structure

```
src/
├── lib.rs      # Core transformation logic (transliterate_char, transform_filename, compute_renames, etc.)
├── main.rs     # CLI (argument parsing, conflict detection, main loop)
tests/
├── cli_tests.rs       # CLI integration tests
├── recursive_tests.rs  # Recursive (-r) integration tests
└── transform_tests.rs # Unit tests
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
- **Linter**: Code MUST be compliant with `clippy` (no warnings allowed).
- **Tests location**: `tests/` directory
- **Test naming**: `tests/test_*.rs` or `tests/*_tests.rs`

---

## CLI Usage

```
rename-simple [OPTIONS] [DIR]

Arguments:
  DIR   Target directory (default: current directory)

Options:
  -n, --dry-run   Show what would be renamed without touching any file
  -d, --dirs      Rename directory names only
  -f, --files     Rename file names only
  -r, --recursive Process directories recursively
  -h, --help      Print this help message
```

---

## Error Handling

- **Conflict detection**: Multiple files that would rename to the same destination are skipped with a warning.
- **Existing destination**: Files that already exist at the target path are skipped with a warning.
- **Hidden files**: Files starting with `.` are ignored.
- **IO errors**: Reported per-file with error count in summary.

---

## Validation & Quality Gate

The Agent must execute and pass the following "Checklist" before proposing a solution or finishing a task:

1. **Format Check**: Run `cargo fmt --all`.
2. **Linting Check**: Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. **Security Audit**: Run `cargo audit` to ensure no vulnerable dependencies are introduced.
4. **Logic Verification**: Run `cargo test` and ensure 100% pass rate.
5. **Documentation Check**: Run `cargo doc --no-deps` to ensure internal links and docstrings are valid.

---

## TDD Workflow

1. **Write failing test first** — In the `tests/` directory.
2. **Run tests** — `cargo test` (validate red phase).
3. **Implement minimal code** — Make the test pass (green phase).
4. **Clean up** — Run `cargo fmt` and `cargo clippy`.
5. **Refactor** — Improve code while keeping tests green.
6. **After 5 iterations**, stop and ask the user for direction.
7. **Final validation** — Run the full Validation & Quality Gate checklist and fix any issues.

---

## Language

- **Code, comments, docstrings, and documentation**: Always in English
- **Communication with user**: Respond in the user's language (French in this case)
