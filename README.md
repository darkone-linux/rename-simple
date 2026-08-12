# rename-simple

[![CI](https://github.com/darkone-linux/rename-simple/actions/workflows/ci.yml/badge.svg)](https://github.com/darkone-linux/rename-simple/actions/workflows/ci.yml)
[![Rustc](https://img.shields.io/badge/rustc-1.85%2B-blue)](https://rust-lang.org)
[![Version](https://img.shields.io/crates/v/rename-simple)](https://crates.io/crates/rename-simple)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow)](./LICENSE)

A small Rust CLI tool that renames files and directories to clean, ASCII-safe slugs.

![rename-simple demo](https://raw.githubusercontent.com/darkone-linux/rename-simple/main/assets/rename-simple.png)

> [!WARNING]
> **Breaking change since 0.4.0.** The directory-scan mode and the
> `-a`/`--all` and `-r`/`--recursive` options have been removed.
> `rename-simple` now operates **only** on the paths you give it, like
> `rename`(1). Use your shell's globbing to select entries:
> `rename-simple *` instead of `rename-simple -a`, and
> `rename-simple **/*.pdf` instead of `-r`.

## What it does

- Transliterates accented and extended Latin characters to ASCII (`é → e`, `ç → c`, `œ → oe`, `ß → ss`…)
- Expands typographic ligatures and compatibility forms (`ﬁ → fi`, fullwidth `Ｆｉｌｅ → file`, superscripts `² → 2`, `™ → tm`, `№ → no`)
- Romanises Greek and Cyrillic letters (`Ελληνικά → ellinika`, `Москва → moskva`)
- Lowercases everything
- Replaces spaces and special characters with `-`; collapses consecutive separators
- Preserves `_`; cleans up `_-` and `-_` sequences to `_`
- Strips leading and trailing `-` / `_` before the extension
- Preserves known compound extensions (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`)
- Keeps extensions separate only when they are ASCII alphanumeric and ≤10 characters
  (e.g. `.tét` → absorbed as `-tet`; `.abcdefghijkl` (12 chars) → absorbed as `-abcdefghijkl`)
- Skips hidden files (`.gitignore`, `.DS_Store`…) and flags naming conflicts
- Optional cleanup fixes: repairs mojibake (`-U`), strips HTML tags and entities (`-H`), or both (`-A`)

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) 1.85+.

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
| `-f`, `--files-only` | Rename files only |
| `-d`, `--dirs-only` | Rename directories only |
| `-U`, `--fix-unicode` | Repair mojibake (UTF-8 misread as Latin-1/CP1252) before renaming |
| `-H`, `--fix-html` | Strip HTML tags and decode HTML entities before renaming |
| `-A`, `--fix-all` | Apply every cleanup fix (currently `-U` + `-H`) |
| `-n`, `--dry-run` | Preview renames without touching any file |
| `-q`, `--quiet` | Print nothing at all |
| `-v`, `--verbose` | Show details of each rename |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

## Examples

### Preview all renames (dry-run)

```bash
$ rename-simple --dry-run ~/Downloads/*
```

```
[R] .: 01_ Introduction au Projet.PDF -> 01_introduction-au-projet.pdf
[R] .: Réunion d'équipe (2024).docx -> reunion-d-equipe-2024.docx
[R] .: backup.TAR.GZ -> backup.tar.gz
[R] .: Café Montréal.jpg -> cafe-montreal.jpg
[R] .: à faire .tét -> a-faire-tet
[R] .: notes.cuicuicuicui -> notes-cuicuicuicui
6 entries matched, 6 entries renamed, 0 error.
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

### Repair damaged names (mojibake, HTML)

Names coming from broken downloads or web scrapers often carry mojibake
(UTF-8 read as Latin-1/CP1252) or HTML markup. The cleanup fixes repair the
name **before** the slug pipeline runs:

```bash
$ rename-simple -A ~/Downloads/*
```

```
[R] .: CafÃ© MontrÃ©al.jpg -> cafe-montreal.jpg
[R] .: Tom &amp; Jerry.mp4 -> tom-jerry.mp4
[R] .: <b>Ã‰tÃ© 2024.pdf -> ete-2024.pdf
3 entries matched, 3 entries renamed, 0 error.
```

`-U`/`--fix-unicode` is all-or-nothing per name: a file that is already
correctly named (`café.jpg`) is never re-decoded, so the fix is safe to apply
everywhere. Double mojibake (`CafÃƒÂ©`) is repaired too. `-H`/`--fix-html`
strips tags and decodes named (`&eacute;`), decimal (`&#233;`) and hex
(`&#xE9;`) entities. Block-level tags (`<br>`, `<p>`, `<div>`, `<li>`,
`<h1>`–`<h6>`, …) are replaced by a space so the surrounding words stay
separated (`data<br>client` → `data-client`), while inline tags (`<b>`, `<i>`,
`<span>`, …) vanish with no gap (`client<b>s` → `clients`). `-A`/`--fix-all`
applies every cleanup fix.

### Verbose output

Verbose mode prints each rename and a summary:

```bash
$ rename-simple -v ~/Downloads/*
```

```
[R] .: 01_ Introduction au Projet.PDF -> 01_introduction-au-projet.pdf
[R] .: Réunion d'équipe (2024).docx -> reunion-d-equipe-2024.docx
[R] .: backup.TAR.GZ -> backup.tar.gz
[R] .: Café Montréal.jpg -> cafe-montreal.jpg
[R] .: à faire .tét -> a-faire-tet
[R] .: notes.cuicuicuicui -> notes-cuicuicuicui
6 entries matched, 6 entries renamed, 0 error.
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
