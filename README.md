# rename-simple

[![CI](https://github.com/darkone-linux/rename-simple/actions/workflows/ci.yml/badge.svg)](https://github.com/darkone-linux/rename-simple/actions/workflows/ci.yml)
[![Rustc](https://img.shields.io/badge/rustc-1.70%2B-blue)](https://rust-lang.org)
[![Version](https://img.shields.io/crates/v/rename-simple)](https://crates.io/crates/rename-simple)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow)](./LICENSE)

A small Rust CLI tool that renames files and directories to clean, ASCII-safe slugs.

![rename-simple demo](assets/rename-simple.png)

## What it does

- Transliterates accented and extended Latin characters to ASCII (`é → e`, `ç → c`, `œ → oe`, `ß → ss`…)
- Lowercases everything
- Replaces spaces and special characters with `-`; collapses consecutive separators
- Preserves `_`; cleans up `_-` and `-_` sequences to `_`
- Strips leading and trailing `-` / `_` before the extension
- Preserves known compound extensions (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`)
- Keeps extensions separate only when they are ASCII alphanumeric and ≤10 characters
  (e.g. `.tét` → absorbed as `-tet`; `.abcdefghijkl` (12 chars) → absorbed as `-abcdefghijkl`)
- Skips hidden files (`.gitignore`, `.DS_Store`…) and flags naming conflicts

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) 1.70+.

```bash
cargo install --path .
```

## Usage

```
rename-simple [OPTIONS] [DIR]
```

You must specify one of `-f`, `-d`, or `-a` to select what to rename.
Running `rename-simple` without any option prints this help.

| Option | Description |
|---|---|
| `DIR` | Directory to process (default: current directory) |
| `-f` | Rename files only |
| `-d` | Rename directories only |
| `-a`, `--all` | Rename both files and directories |
| `-r`, `--recursive` | Process subdirectories recursively |
| `-n`, `--dry-run` | Preview renames without touching any file |
| `-v`, `--verbose` | Show details of each rename |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

## Examples

### Preview all renames (dry-run)

```bash
$ rename-simple -a --dry-run ~/Downloads
```

```
  01_ Introduction au Projet.PDF  →  01-introduction-au-projet.pdf
  Réunion d'équipe (2024).docx    →  reunion-d-equipe-2024.docx
  backup.TAR.GZ                   →  backup.tar.gz
  Café Montréal.jpg               →  cafe-montreal.jpg
  à faire .tét                    →  a-faire-tet
  notes.cuicuicuicui              →  notes-cuicuicuicui
```

### Rename files only

```bash
$ rename-simple -f ~/Downloads
```

Directories are left untouched; only files are renamed.

### Rename directories only

```bash
$ rename-simple -d ~/Projects
```

Files are left untouched; only directories are renamed.

### Rename everything

```bash
$ rename-simple -a ~/Downloads
```

### Recursive processing

```bash
$ rename-simple -a --recursive ~/Documents
```

Renames files and directories at every level of the tree.

### Verbose output

```bash
$ rename-simple -a -v ~/Downloads
```

```
Directory: /home/user/Downloads

  01_ Introduction au Projet.PDF  →  01-introduction-au-projet.pdf
  Réunion d'équipe (2024).docx    →  reunion-d-equipe-2024.docx
  Café Montreal.jpg               →  cafe-montreal.jpg

3 entry/entries renamed, 0 error(s).
```

## Tips

If you use `-a` most of the time, a shell alias saves a few keystrokes:

```bash
alias rsa='rename-simple -a'
```

Add this line to your `~/.bashrc` or `~/.zshrc`, then:

```bash
rsa ~/Downloads          # rename everything
rsa -r ~/Documents       # rename everything, recursively
rsa -n ~/Downloads       # dry-run preview
```

## Running the tests

```bash
cargo test
```

## Known limitations (TODO)

### Recursion can descend into the wrong sibling on a name conflict

When two sibling directories collide after transformation, the rename is
correctly skipped (no data is overwritten), but the recursive descent that
follows uses the **destination name** to locate the renamed dir on disk.
If the destination already exists as a different directory in the same
parent, recursion enters that pre-existing directory instead of the
original one.

Concrete case:

```
parent/
├── Cible/                 ← would rename to "cible" but blocked (conflict)
│   └── fichier.txt        ← never visited
└── cible/                 ← pre-existing, gets visited twice
```

`effective_subdir_path` in `src/main.rs` resolves both `Cible` and `cible`
to `parent/cible`, so the file inside `Cible/` is silently skipped under
`-r`. No data is lost, but recursion coverage is incomplete.

**Workaround**: rename one of the conflicting directories manually before
running `-r`, or run without `-r` first to surface the conflict, then
re-run after resolving it.

**Tracking**: see the audit notes from v0.2.4 — fix candidates include
detecting the case explicitly in `effective_subdir_path` and falling back
to the original path when the destination existed before the run.

## License

MIT
