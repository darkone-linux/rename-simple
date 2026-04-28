# The `rename-simple` cool tool

A small Rust CLI tool that renames files in a directory to clean, ASCII-safe slugs.

```
"   01_ Une     chaîne de      CARACtères.pdf" -> "01_une-chaine-de-caracteres.pdf"
```

## What it does

- Transliterates accented and extended Latin characters to ASCII (`é → e`, `ç → c`, `œ → oe`, `ß → ss`…)
- Lowercases everything
- Replaces spaces and special characters with `-`; collapses consecutive separators
- Preserves `_`; cleans up `_-` and `-_` sequences to `_`
- Strips leading and trailing `-` / `_` before the extension
- Preserves known compound extensions (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`)
- Skips hidden files (`.gitignore`, `.DS_Store`…) and flags naming conflicts

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) 1.70+.

```bash
cargo install --path .
```

## Usage

```
rename-files [OPTIONS] [DIR]
```

| Option | Description |
|---|---|
| `DIR` | Directory to process (default: current directory) |
| `-n`, `--dry-run` | Preview renames without touching any file |
| `-h`, `--help` | Print help |

> **Coming soon:** `-r` for recursive processing.

## Example output

```
$ rename-files --dry-run ~/Downloads

Directory: /home/user/Downloads

  01_ Introduction au Projet.PDF  →  01_introduction-au-projet.pdf
  Réunion d'équipe (2024).docx    →  reunion-d-equipe-2024.docx
  backup.TAR.GZ                   →  backup.tar.gz
  ⚠  CONFLICT – skipping 'note (1).txt': destination 'note-1.txt' already exists

3 file(s) would be renamed.
```

## Running the tests

```bash
cargo test
```
