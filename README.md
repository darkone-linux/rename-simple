# rename-simple

[![CI](https://github.com/darkone-linux/rename-simple/actions/workflows/ci.yml/badge.svg)](https://github.com/darkone-linux/rename-simple/actions/workflows/ci.yml)
[![Rustc](https://img.shields.io/badge/rustc-1.70%2B-blue)](https://rust-lang.org)
[![Version](https://img.shields.io/crates/v/rename-simple)](https://crates.io/crates/rename-simple)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow)](./LICENSE)

A small Rust CLI tool that renames files and directories to clean, ASCII-safe slugs.

![rename-simple demo](assets/rename-simple.gif)

> [!WARNING]
> **Breaking change since 0.4.0.** The directory-scan mode and the
> `-a`/`--all` and `-r`/`--recursive` options have been removed.
> `rename-simple` now operates **only** on the paths you give it, like
> `rename`(1). Use your shell's globbing to select entries:
> `rename-simple *` instead of `rename-simple -a`, and
> `rename-simple **/*.pdf` instead of `-r`.

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
# From crates.io
cargo install rename-simple

# Or from source
cargo install --path .
```

## Usage

```
rename-simple [OPTIONS] <files>...
```

Each argument is an entry renamed **itself**, like the traditional `rename`
command — globbing is left to the shell (`rename-simple *.jpg`, or
`rename-simple dir/**/*.pdf` with zsh / bash `globstar`). Files and directories
are both renamed by default; `-f` / `-d` act as a type filter. With no
arguments, `rename-simple` prints this help.

| Option | Description |
|---|---|
| `<files>...` | Entries to rename (files and/or directories) |
| `-f` | Rename files only |
| `-d` | Rename directories only |
| `-n`, `--dry-run` | Preview renames without touching any file |
| `-v`, `--verbose` | Show details of each rename |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

## Examples

### Preview all renames (dry-run)

```bash
$ rename-simple --dry-run ~/Downloads/*
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
$ rename-simple -f ~/Downloads/*
```

Directories among the arguments are left untouched; only files are renamed.

### Rename directories only

```bash
$ rename-simple -d ~/Projects/*
```

Files among the arguments are left untouched; only directories are renamed.

### Rename everything

```bash
$ rename-simple ~/Downloads/*
```

### Process a whole tree

With a shell that supports recursive globs you can select entries at any depth:

```bash
$ rename-simple ~/Documents/**/*.pdf
```

### Verbose output

Verbose mode prints each rename and a summary:

```bash
$ rename-simple -v ~/Downloads/*
```

```
  01_ Introduction au Projet.PDF  →  01-introduction-au-projet.pdf
  Réunion d'équipe (2024).docx    →  reunion-d-equipe-2024.docx
  Café Montréal.jpg               →  cafe-montreal.jpg
  …
3 entry/entries renamed, 0 error(s).
```

## Tips

Since the directory-scan mode is gone, "rename everything in the current
directory" is now just `rename-simple *`. A shell alias makes it a habit:

```bash
alias rsa='rename-simple *'
```

Add this line to your `~/.bashrc` or `~/.zshrc`, then:

```bash
rsa                      # rename everything in the current directory
rename-simple -n *       # dry-run preview
rename-simple **/*.pdf   # rename every PDF in the tree (zsh / bash globstar)
```

## Running the tests

```bash
cargo test
```

## License

MIT
