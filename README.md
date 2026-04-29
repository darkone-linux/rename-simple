# rename-simple

A small Rust CLI tool that renames files and directories to clean, ASCII-safe slugs.

```
"   01_ Une     chaîne de      CARACtères.pdf" → "01-une-chaine-de-caracteres.pdf"
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
rename-simple [OPTIONS] [DIR]
```

| Option | Description |
|---|---|
| `DIR` | Directory to process (default: current directory) |
| `-f`, `--files` | Rename files only |
| `-d`, `--dirs` | Rename directories only |
| `-r`, `--recursive` | Process subdirectories recursively |
| `-n`, `--dry-run` | Preview renames without touching any file |
| `-h`, `--help` | Print help |

## Examples

### Basic usage

```bash
$ rename-simple --dry-run ~/Downloads

Directory: /home/user/Downloads

  01_ Introduction au Projet.PDF  →  01-introduction-au-projet.pdf
  Réunion d'équipe (2024).docx    →  reunion-d-equipe-2024.docx
  backup.TAR.GZ                   →  backup.tar.gz
  Café Montréal.jpg               →  cafe-montreal.jpg

4 file(s) would be renamed.
```

### Rename only directories

```bash
$ rename-simple --dirs ~/Projects

  Mes Documents/           →  mes-documents/
  workspace/               →  workspace/

2 directory(ies) would be renamed.
```

### Actual rename (without dry-run)

```bash
$ rename-simple ~/Downloads

  01_ Introduction au Projet.PDF  →  01-introduction-au-projet.pdf
  Réunion d'équipe (2024).docx    →  reunion-d-equipe-2024.docx
  Café Montreal.jpg               →  cafe-montreal.jpg

3 file(s) renamed, 0 error(s).
```

### Recursive processing

```bash
$ rename-simple --recursive ~/Documents

Directory: /home/user/Documents

  Rapport 2024.pdf               →  rapport-2024.pdf
  Notes/                         →  notes/

Directory: /home/user/Documents/Notes

  Réunion'équipe.md               →  reunion-equipe.md
  Café Info.txt                  →  cafe-info.txt

5 entry/entries would be renamed, 0 error(s).
```

## Running the tests

```bash
cargo test
```

## License

MIT
