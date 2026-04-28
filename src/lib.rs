use std::path::{Path, PathBuf};
use std::fs;

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
            '0' => "0", '1' => "1", '2' => "2", '3' => "3", '4' => "4",
            '5' => "5", '6' => "6", '7' => "7", '8' => "8", '9' => "9",
            _   => "-",
        };
    }
    if c.is_ascii_alphabetic() {
        return match c.to_ascii_lowercase() {
            'a' => "a", 'b' => "b", 'c' => "c", 'd' => "d", 'e' => "e",
            'f' => "f", 'g' => "g", 'h' => "h", 'i' => "i", 'j' => "j",
            'k' => "k", 'l' => "l", 'm' => "m", 'n' => "n", 'o' => "o",
            'p' => "p", 'q' => "q", 'r' => "r", 's' => "s", 't' => "t",
            'u' => "u", 'v' => "v", 'w' => "w", 'x' => "x", 'y' => "y",
            'z' => "z", _ => "-",
        };
    }
    if c == '_' {
        return "_";
    }

    // Extended Latin — both cases handled in a single arm
    match c {
        // A
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' |
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å'          => "a",
        'Æ' | 'æ'                                     => "ae",
        // C
        'Ç' | 'ç'                                     => "c",
        // D
        'Ð' | 'ð'                                     => "d",
        // E
        'È' | 'É' | 'Ê' | 'Ë' |
        'è' | 'é' | 'ê' | 'ë'                         => "e",
        // I
        'Ì' | 'Í' | 'Î' | 'Ï' |
        'ì' | 'í' | 'î' | 'ï'                         => "i",
        // N
        'Ñ' | 'ñ'                                     => "n",
        // O
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' |
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø'            => "o",
        'Œ' | 'œ'                                     => "oe",
        // S
        'ß'                                           => "ss",
        // T (Thorn)
        'Þ' | 'þ'                                     => "th",
        // U
        'Ù' | 'Ú' | 'Û' | 'Ü' |
        'ù' | 'ú' | 'û' | 'ü'                         => "u",
        // Y
        'Ý' | 'Ÿ' | 'ý' | 'ÿ'                        => "y",
        // Z
        'Ź' | 'Ż' | 'Ž' | 'ź' | 'ż' | 'ž'           => "z",
        // Anything else is a separator
        _                                             => "-",
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

/// Remove leading and trailing `-` or `_` characters.
fn trim_separators(s: &str) -> String {
    s.trim_matches(|c| c == '-' || c == '_').to_owned()
}

/// Transform a filename **stem** (without extension) into a clean ASCII slug.
///
/// Pipeline:
/// 1. Transliterate every character.
/// 2. Collapse consecutive dashes.
/// 3. Remove `-` adjacent to `_`.
/// 4. Trim leading / trailing `-` and `_`.
pub fn transform_stem(stem: &str) -> String {
    let raw: String = stem.chars().map(transliterate_char).collect();
    let collapsed = collapse_dashes(&raw);
    let fixed = fix_underscore_dash(&collapsed);
    trim_separators(&fixed)
}

/// Transform a full filename (stem + extension).
///
/// The extension is only lowercased (extensions are always plain ASCII in
/// practice). The stem goes through the full `transform_stem` pipeline.
/// Hidden files (names starting with `.`) are returned unchanged.
pub fn transform_filename(filename: &str) -> String {
    // Leave hidden files alone
    if filename.starts_with('.') {
        return filename.to_owned();
    }

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

    let new_stem = transform_stem(stem);

    if new_stem.is_empty() {
        return format!("unnamed{}", ext);
    }

    format!("{}{}", new_stem, ext)
}

// ─────────────────────────────────────────────────────────────────────────────
// Filesystem operations
// ─────────────────────────────────────────────────────────────────────────────

/// A single rename operation computed (but not yet applied).
#[derive(Debug, Clone)]
pub struct RenameOp {
    pub from: PathBuf,
    pub to:   PathBuf,
}

/// Walk `dir` (non-recursively) and return the list of renames to perform.
///
/// Files whose name already matches the transformed result are skipped.
/// Hidden files (names starting with `.`) are skipped.
/// Directories are skipped (recursive mode is reserved for a future `-r` flag).
pub fn compute_renames(dir: &Path) -> Result<Vec<RenameOp>, std::io::Error> {
    let mut ops = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path  = entry.path();

        if !path.is_file() {
            continue;
        }

        let original = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_owned(),
            None       => continue,
        };

        // Skip hidden files
        if original.starts_with('.') {
            continue;
        }

        let renamed = transform_filename(&original);

        if renamed == original {
            continue; // nothing to do
        }

        ops.push(RenameOp {
            from: path.clone(),
            to:   dir.join(&renamed),
        });
    }

    Ok(ops)
}
