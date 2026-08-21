# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-08-21

Renaming behaviour is unchanged; what changes is what happens when the cleaned
name is already taken. Such a clash was reported as a bare
`File name already exists`, whatever sat at the destination — including the
common case of a file already renamed on an earlier run, sitting next to an
untouched copy of itself. The diagnostic now says which of the two situations
it is, and the new `-D` flag clears the harmless one.

### Added
- `-D`/`--delete-duplicates`: when the source and the existing destination are
  two regular files with byte-for-byte identical content, the rename would only
  produce a copy of what is already there, so the redundant source is deleted
  and reported as a warning (`[W] dir: source -> Identical duplicate, source
  deleted`). Deliberately **not** implied by `-A`/`--fix-all`, which only
  repairs names: deletion is the single irreversible operation of this program
  and always requires an explicit flag. Under `--dry-run` the deletion is
  announced, never performed.
- `[W]` output category, on standard error like `[E]`, silenced by `-q`.
  The summary gains a `N duplicates removed` segment, printed only when
  non-zero so the usual one-line report is unchanged.
- Public API: `compare_entries` and `EntryMatch` (`SameEntry`, `Identical`,
  `Different`, `NotComparable`). Comparison is exact rather than digest-based:
  entry identity first, then size, then a streamed byte-for-byte comparison, so
  differing files bail out on the first divergent block, no file is ever fully
  loaded in memory, and no hash collision can be mistaken for equality.

### Changed
- A name clash between two regular files with differing content now reports
  `File name already exists (2 different files)`. When the contents are
  identical, the error names the flag that would clear it:
  `File name already exists (identical duplicate, use -D to delete the source)`.
  Both remain errors and touch nothing; the exit status is unchanged.
- Two names for the same entry (hard link, symlink to the destination, or a
  case-insensitive filesystem folding both names together) keep the plain
  `File name already exists` and are never deduplicated, with or without `-D`:
  deleting the source there would destroy the last copy rather than a duplicate.
  Directories, and any other non-regular entry, are never compared nor deleted.

## [0.6.0] - 2026-08-12

No change to the renaming behaviour: this release is packaging and tooling. It
is a minor rather than a patch because the declared minimum Rust version moves
from 1.70 to 1.85.

### Added
- **Nix flake** (`flake.nix`): exposes `packages.<system>.{default,rename-simple}`,
  `apps.<system>.default` for `nix run`, and `devShells.<system>.default`.
  Supports `x86_64-linux` and `aarch64-linux`. The package is built from source
  in the Nix sandbox with `rustPlatform.buildRustPackage`, replacing a
  derivation that copied a host-built, dynamically linked binary into the store
  and was therefore neither reproducible nor portable.
- `LICENSE` file. The MIT license was declared in `Cargo.toml` and linked from
  the README badge, but the text was absent from the repository and from the
  published crate.
- CI job checking the crate builds on the declared MSRV, reading `rust-version`
  straight from `Cargo.toml` so the two can never disagree.
- CI job building the Nix flake package.
- Dependabot configuration for cargo, GitHub Actions and Nix flake inputs.

### Changed
- **Minimum supported Rust version raised from 1.70 to 1.85.** This documents
  reality rather than dropping support: the locked `clap` and `assert_cmd`
  already declared `rust-version = "1.85"`, so the crate could not build on
  1.70–1.84. `resolver = "3"` was added so cargo honours the floor during
  dependency resolution — the edition-2021 default resolver ignores
  `rust-version`, which is why the mismatch went unnoticed.
- Dependencies updated: 17 crates, including `clap` 4.6.1 → 4.6.6,
  `regex` 1.12.4 → 1.13.1 and `syn` 2.0.118 → 3.0.3.
- The published crate no longer ships development tooling (`Justfile`,
  `shell.nix`, `.github/`, agent instructions and internal notes).
- `just package nix` delegates to `nix build .#rename-simple`.

### Fixed
- The CI formatting gate ran `cargo fmt --all` in rewrite mode instead of
  `--check`, so it always exited 0 and could never fail a build; the audit and
  documentation steps were missing entirely. CI now invokes `just ci`, the same
  gate used locally.
- The README screenshot rendered as a broken image on crates.io: it was
  referenced by relative path while `assets/` is excluded from the tarball. It
  is now an absolute URL.
- The Nix dev shell overrode the nixpkgs revision pinned in `shell.nix`, so
  `nix develop` and `nix-shell` could drift apart. Both now resolve to the same
  pinned toolchain.
- The Nix package did not install the man page.

## [0.5.1] - 2026-07-25

### Changed
- Dependencies refreshed to their latest compatible versions (`cargo update`).
- `split_extension` no longer allocates a lowercased copy of the whole filename
  nor a per-extension format string on every call: the known compound
  extensions (`.tar.gz`, …) are now matched by a case-insensitive comparison of
  the trailing bytes. Behaviour is unchanged.

### Added
- Regression tests for the HTML entity decoder: the body-length cap
  (`MAX_ENTITY_LEN`) and out-of-range / `u32`-overflowing numeric entities.

## [0.5.0] - 2026-07-02

### Added
- **`-U`/`--fix-unicode`**: repairs mojibake before renaming — names whose
  UTF-8 bytes were wrongly decoded as Latin-1 or Windows-1252 (`CafÃ©.txt` is
  treated as `Café.txt` and becomes `cafe.txt`; CP1252 artefacts like `â€™`
  for `’` are handled too). The repair is all-or-nothing per name: anything
  not fully re-decodable is left untouched, so correctly named files are never
  corrupted. Double/triple mojibake (`CafÃƒÂ©`) is repaired iteratively.
  New public `fix_unicode` function.
- **`-H`/`--fix-html`**: strips HTML tags and decodes HTML entities before
  renaming (`<b>Tom &amp; Jerry.mp4` → `tom-jerry.mp4`). Block-level tags
  (`<br>`, `<p>`, `<div>`, `<li>`, `<h1>`–`<h6>`, …) are replaced by a space so
  the surrounding words stay separated (`data<br>client` → `data-client`),
  while inline tags (`<b>`, `<i>`, `<span>`, …) are removed with no gap
  (`client<b>s` → `clients`). Supports named (`&eacute;`), decimal (`&#233;`)
  and hexadecimal (`&#xE9;`) entities; anything that does not parse as a tag or
  entity is kept verbatim. New public `fix_html` function.
- **`-A`/`--fix-all`**: applies every cleanup fix (currently `-U` + `-H`);
  future cleanup passes will be folded into it. New public `CleanupOptions`
  struct and `transform_filename_with` / `transform_dirname_with` /
  `plan_entry_with` functions carrying the options through the library API
  (the historical functions keep their exact behaviour).
- **Greek and Cyrillic romanisation**: Greek (`Ελληνικά` → `ellinika`,
  `θ` → `th`, `ψ` → `ps`) and Cyrillic — Russian plus common
  Ukrainian/Belarusian letters (`Москва` → `moskva`, `ж` → `zh`,
  `щ` → `shch`) — instead of collapsing to dashes.
- **More Latin transliterations**: capital sharp s `ẞ`, eng `Ŋ/ŋ` → `ng`,
  schwa `Ə/ə` and open vowels `Ɛ/ɛ`/`Ɔ/ɔ`, f-hook `ƒ`, kra `ĸ`, and the
  Serbo-Croatian digraph code points `Ǆ/Ǉ/Ǌ/Ǳ` → `dz`/`lj`/`nj`/`dz`.
- 55 new tests covering the transliteration additions, both cleanup fixes and
  the new CLI flags (unit + end-to-end).

### Changed
- The slug pipeline now normalises to **NFKD** (compatibility decomposition)
  instead of NFD: typographic ligatures (`ﬁle` → `file`), fullwidth forms
  (`Ｆｉｌｅ０１` → `file01`), superscripts/subscripts (`x²` → `x2`), `™` → `tm`,
  `№` → `no` and vulgar fractions (`½` → `1-2`) now transliterate instead of
  collapsing to dashes.
- Man page normalised: `.TH` synced with the package version, synopsis
  enumerates every short flag, options use standard bold/roman formatting,
  `-V`/`--version` documented, exit status clarified (per-entry failures do
  not change it), and the new cleanup options documented with examples.
  `groff` renders it warning-free.
- Internal cleanup: the duplicated dash/underscore collapsing helpers merged
  into a single `collapse_runs` function.

## [0.4.1] - 2026-07-02

### Added
- `--files-only` long option for `-f` and `--dirs-only` long option for `-d`.

### Changed
- Man page and README updated to document the new long options; README examples
  now show the real `[R]`/`[E]` output format.

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
- Each output line now carries a grey `dir:` prefix locating where the entry
  lives (`.` for the current directory, the relative sub-path for nested
  entries), e.g. `[R] Sub Dir: Photo.jpg -> photo.jpg`.
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

### Dependencies
- Refreshed the lockfile: `bstr` 1.12.1 → 1.12.3, `getrandom` 0.4.2 → 0.4.3,
  `quote` 1.0.45 → 1.0.46, `syn` 2.0.117 → 2.0.118.
- Bumping `getrandom` drops the transitive `wit-*` / `wasip2` / `wasip3` /
  `wasm-*` / `anyhow` / `serde` toolchain pulled in via `tempfile`, shrinking the
  dependency tree from 83 to 55 crates.
- Clears `RUSTSEC-2026-0190` (unsoundness in `anyhow::Error::downcast_mut`): the
  affected `anyhow` 1.0.102 is no longer part of the tree.

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