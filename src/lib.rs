use std::fs;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Character transliteration
// ─────────────────────────────────────────────────────────────────────────────

/// Transliterate a single Unicode character to its ASCII equivalent(s).
///
/// - ASCII alphanumerics are lowercased and returned as-is.
/// - `_` is preserved.
/// - Accented/special letters are mapped to their base ASCII form.
/// - Everything else (spaces, punctuation, unknown chars…) returns `"-"`,
///   which will act as a separator marker in the pipeline.
pub fn transliterate_char(c: char) -> &'static str {
    // Fast path: plain ASCII
    if c.is_ascii_digit() {
        return match c {
            '0' => "0",
            '1' => "1",
            '2' => "2",
            '3' => "3",
            '4' => "4",
            '5' => "5",
            '6' => "6",
            '7' => "7",
            '8' => "8",
            '9' => "9",
            _ => "-",
        };
    }
    if c.is_ascii_alphabetic() {
        return match c.to_ascii_lowercase() {
            'a' => "a",
            'b' => "b",
            'c' => "c",
            'd' => "d",
            'e' => "e",
            'f' => "f",
            'g' => "g",
            'h' => "h",
            'i' => "i",
            'j' => "j",
            'k' => "k",
            'l' => "l",
            'm' => "m",
            'n' => "n",
            'o' => "o",
            'p' => "p",
            'q' => "q",
            'r' => "r",
            's' => "s",
            't' => "t",
            'u' => "u",
            'v' => "v",
            'w' => "w",
            'x' => "x",
            'y' => "y",
            'z' => "z",
            _ => "-",
        };
    }
    if c == '_' {
        return "_";
    }

    // Extended Latin — both cases handled in a single arm
    match c {
        // A
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
        'Æ' | 'æ' => "ae",
        // C
        'Ç' | 'ç' => "c",
        // D
        'Ð' | 'ð' => "d",
        // E
        'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => "e",
        // I
        'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => "i",
        // N
        'Ñ' | 'ñ' => "n",
        // O
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
        'Œ' | 'œ' => "oe",
        // S
        'ß' => "ss",
        // T (Thorn)
        'Þ' | 'þ' => "th",
        // U
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => "u",
        // Y
        'Ý' | 'Ÿ' | 'ý' | 'ÿ' => "y",
        // Z
        'Ź' | 'Ż' | 'Ž' | 'ź' | 'ż' | 'ž' => "z",
        // Anything else is a separator
        _ => "-",
    }
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
/// 1. Transliterate every character.
/// 2. Collapse consecutive `-`.
/// 3. Remove `-` adjacent to `_` (`_-` → `_`, `-_` → `_`).
/// 4. Collapse consecutive `_` (step 3 can produce `__` from e.g. `_-_`).
/// 5. Trim leading / trailing `-` and `_`.
pub fn transform_stem(stem: &str) -> String {
    let raw: String = stem.chars().map(transliterate_char).collect();
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
        let suffix = format!(".{}", double_ext);
        if lower.ends_with(&suffix) {
            let stem = &filename[..filename.len() - suffix.len()];
            return (stem, suffix);
        }
    }

    // Standard single-extension split via Path
    let path = Path::new(filename);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()))
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    (stem, ext)
}

/// Transform a full filename (stem + extension).
///
/// Known compound extensions (e.g. `.tar.gz`) are preserved as a unit.
/// All other extensions are simply lowercased.
/// The stem goes through the full `transform_stem` pipeline.
/// Hidden files (names starting with `.`) are returned unchanged.
pub fn transform_filename(filename: &str) -> String {
    // Leave hidden files alone
    if filename.starts_with('.') {
        return filename.to_owned();
    }

    let (stem, ext) = split_extension(filename);
    let new_stem = transform_stem(stem);

    if new_stem.is_empty() {
        return format!("unnamed{}", ext);
    }

    format!("{}{}", new_stem, ext)
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
