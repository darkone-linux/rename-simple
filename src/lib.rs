use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

// ─────────────────────────────────────────────────────────────────────────────
// Character transliteration
// ─────────────────────────────────────────────────────────────────────────────

/// Lowercase ASCII letters and digits as static `&str` slices.
/// Indexed by `(0..=9, a..=z)` for table-lookup transliteration.
const ASCII_LOWER_TABLE: [&str; 36] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i",
    "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
];

/// Map an ASCII alphanumeric character to its lowercase static slice.
/// Returns `"-"` if `c` is not ASCII alphanumeric (defensive default).
fn ascii_alnum_to_lower(c: char) -> &'static str {
    let lower = c.to_ascii_lowercase();
    if lower.is_ascii_digit() {
        ASCII_LOWER_TABLE[(lower as u8 - b'0') as usize]
    } else if lower.is_ascii_lowercase() {
        ASCII_LOWER_TABLE[(lower as u8 - b'a' + 10) as usize]
    } else {
        "-"
    }
}

/// Non-decomposable Latin letters that need an explicit ASCII expansion.
///
/// Unicode NFD does not break these into base + combining marks (they have no
/// canonical decomposition), so they would otherwise fall through to `"-"`.
fn special_latin(c: char) -> Option<&'static str> {
    match c {
        'Æ' | 'æ' => Some("ae"),
        'Œ' | 'œ' => Some("oe"),
        'ß' => Some("ss"),
        'Þ' | 'þ' => Some("th"),
        'Ø' | 'ø' => Some("o"),
        'Ł' | 'ł' => Some("l"),
        'Đ' | 'đ' | 'Ð' | 'ð' => Some("d"),
        'Ħ' | 'ħ' => Some("h"),
        'Ŧ' | 'ŧ' => Some("t"),
        'Ĳ' | 'ĳ' => Some("ij"),
        'ı' => Some("i"),
        _ => None,
    }
}

/// Transliterate a single Unicode character to its ASCII equivalent(s).
///
/// Pipeline:
/// 1. Combining marks (e.g. U+0301 acute) are dropped (`""`).
/// 2. ASCII alphanumerics are lowercased.
/// 3. `_` is preserved.
/// 4. A small special map covers non-decomposable Latin letters
///    (`Æ`, `Œ`, `ß`, `Þ`, `Ø`, `Ł`, `Đ`/`Ð`, `Ħ`, `Ŧ`, `Ĳ`, `ı`).
/// 5. Otherwise the char is NFD-decomposed; if the base is an ASCII letter,
///    its lowercase form is returned. This covers the entire Latin Extended-A
///    block (Polish, Czech, Romanian, Turkish dotted/dotless I, etc.).
/// 6. Everything else (spaces, punctuation, CJK, emoji…) returns `"-"`.
#[must_use]
pub fn transliterate_char(c: char) -> &'static str {
    if is_combining_mark(c) {
        return "";
    }
    if c.is_ascii_alphanumeric() {
        return ascii_alnum_to_lower(c);
    }
    if c == '_' {
        return "_";
    }
    if let Some(s) = special_latin(c) {
        return s;
    }

    // NFD fallback: covers any precomposed Latin letter whose canonical
    // decomposition starts with an ASCII letter (À, é, Č, Ą, ș, İ…).
    if let Some(base) = c.nfd().find(|x| !is_combining_mark(*x)) {
        if base != c && base.is_ascii_alphabetic() {
            return ascii_alnum_to_lower(base);
        }
    }
    "-"
}

// ─────────────────────────────────────────────────────────────────────────────
// String transformation pipeline
// ─────────────────────────────────────────────────────────────────────────────

/// Collapse any run of consecutive `-` characters into a single `-`.
fn collapse_dashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_dash {
                out.push('-');
            }
            prev_dash = true;
        } else {
            prev_dash = false;
            out.push(c);
        }
    }
    out
}

/// Remove `-` adjacent to `_`:
///   `_-` → `_`  and  `-_` → `_`
///
/// Repeated in a loop until no pattern remains (handles chains like `_-_-`).
fn fix_underscore_dash(s: &str) -> String {
    let mut current = s.to_owned();
    loop {
        let next = current.replace("_-", "_").replace("-_", "_");
        if next == current {
            break;
        }
        current = next;
    }
    current
}

/// Collapse any run of consecutive `_` characters into a single `_`.
///
/// Needed because removing `-` between two `_` (e.g. `_-_`) leaves `__`.
fn collapse_underscores(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c == '_' {
            if !prev_underscore {
                out.push('_');
            }
            prev_underscore = true;
        } else {
            prev_underscore = false;
            out.push(c);
        }
    }
    out
}

/// Remove leading and trailing `-` or `_` characters.
fn trim_separators(s: &str) -> String {
    s.trim_matches(|c| c == '-' || c == '_').to_owned()
}

/// Transform a filename **stem** (without extension) into a clean ASCII slug.
///
/// Pipeline:
/// 1. Normalise the input to NFD so accented letters split into base + marks.
///    This makes the transformation idempotent regardless of whether the
///    input filename was stored as NFC (`café`) or NFD (`cafe\u{0301}`).
/// 2. Transliterate every character (combining marks become empty).
/// 3. Collapse consecutive `-`.
/// 4. Remove `-` adjacent to `_` (`_-` → `_`, `-_` → `_`).
/// 5. Collapse consecutive `_` (step 4 can produce `__` from e.g. `_-_`).
/// 6. Trim leading / trailing `-` and `_`.
#[must_use]
pub fn transform_stem(stem: &str) -> String {
    let raw: String = stem.nfd().map(transliterate_char).collect();
    let collapsed = collapse_dashes(&raw);
    let fixed = fix_underscore_dash(&collapsed);
    let fixed = collapse_underscores(&fixed);
    trim_separators(&fixed)
}

/// Known compound extensions that must be kept together.
/// Stored and matched in lowercase — add new ones here as needed.
const DOUBLE_EXTENSIONS: &[&str] = &["tar.gz", "tar.bz2", "tar.xz", "tar.zst"];

/// Extract a compound extension if the filename ends with one of the known
/// double extensions (case-insensitive), and return `(stem, ".compound.ext")`.
/// Falls back to the standard single-extension split otherwise.
fn split_extension(filename: &str) -> (&str, String) {
    let lower = filename.to_ascii_lowercase();

    for &double_ext in DOUBLE_EXTENSIONS {
        let suffix = format!(".{double_ext}");
        if lower.ends_with(&suffix) {
            let stem = &filename[..filename.len() - suffix.len()];
            return (stem, suffix);
        }
    }

    // Standard single-extension split via Path.
    // An extension is only valid when every character is ASCII alphanumeric
    // AND the total length does not exceed 10.  Non-ASCII characters (accents,
    // spaces, punctuation, …) or a length > 10 cause the candidate extension
    // to be re-absorbed into the stem so it goes through the full
    // transliteration pipeline.
    let path = Path::new(filename);
    let ext_str = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let valid_ext = !ext_str.is_empty()
        && ext_str.chars().all(|c| c.is_ascii_alphanumeric())
        && ext_str.len() <= 10;

    if valid_ext {
        let ext = format!(".{}", ext_str.to_ascii_lowercase());
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);
        (stem, ext)
    } else {
        (filename, String::new())
    }
}

/// Transform a full filename (stem + extension).
///
/// Known compound extensions (e.g. `.tar.gz`) are preserved as a unit.
/// All other extensions are simply lowercased.
/// The stem goes through the full `transform_stem` pipeline.
/// Hidden files (names starting with `.`) are returned unchanged.
#[must_use]
pub fn transform_filename(filename: &str) -> String {
    // Leave hidden files alone
    if filename.starts_with('.') {
        return filename.to_owned();
    }

    let (stem, ext) = split_extension(filename);
    let new_stem = transform_stem(stem);

    if new_stem.is_empty() {
        return format!("unnamed{ext}");
    }

    format!("{new_stem}{ext}")
}

// ─────────────────────────────────────────────────────────────────────────────
// Filesystem operations
// ─────────────────────────────────────────────────────────────────────────────

/// Controls which filesystem entries are processed by `compute_renames`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameTarget {
    /// Rename both files and directories (default).
    All,
    /// Rename files only (`-f`).
    FilesOnly,
    /// Rename directories only (`-d`).
    DirsOnly,
}

/// A single rename operation computed (but not yet applied).
#[derive(Debug, Clone)]
pub struct RenameOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Walk `dir` (non-recursively) and return the list of renames to perform.
///
/// The `target` parameter controls whether files, directories, or both are
/// considered. Hidden entries (names starting with `.`) are always skipped.
pub fn compute_renames(dir: &Path, target: RenameTarget) -> Result<Vec<RenameOp>, std::io::Error> {
    let mut ops = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let is_file = path.is_file();
        let is_dir = path.is_dir();

        // Filter by target
        let include = match target {
            RenameTarget::All => is_file || is_dir,
            RenameTarget::FilesOnly => is_file,
            RenameTarget::DirsOnly => is_dir,
        };
        if !include {
            continue;
        }

        let original = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_owned(),
            None => continue,
        };

        // Skip hidden entries
        if original.starts_with('.') {
            continue;
        }

        // Directories have no extension: transform_filename degrades to
        // transform_stem cleanly when there is no dot in the name.
        let renamed = transform_filename(&original);

        if renamed == original {
            continue; // nothing to do
        }

        ops.push(RenameOp {
            from: path.clone(),
            to: dir.join(&renamed),
        });
    }

    Ok(ops)
}
