use std::fs;
use std::io::{self, BufRead, BufReader};
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

/// Non-decomposable Latin letters, digraphs, typographic ligatures and a few
/// symbols that need an explicit multi-character ASCII expansion.
///
/// Unicode NFD does not break these into base + combining marks (they have no
/// canonical decomposition), so they would otherwise fall through to `"-"`.
/// The `ﬁ`-style ligatures, `™` and `№` do have a compatibility (NFKD)
/// decomposition, but the single-char fallback can only return the first base
/// character — the explicit map keeps `transliterate_char` exact for them.
fn special_latin(c: char) -> Option<&'static str> {
    match c {
        'Æ' | 'æ' => Some("ae"),
        'Œ' | 'œ' => Some("oe"),
        'ß' | 'ẞ' => Some("ss"),
        'Þ' | 'þ' => Some("th"),
        'Ø' | 'ø' | 'Ɔ' | 'ɔ' => Some("o"),
        'Ł' | 'ł' => Some("l"),
        'Đ' | 'đ' | 'Ð' | 'ð' => Some("d"),
        'Ħ' | 'ħ' => Some("h"),
        'Ŧ' | 'ŧ' => Some("t"),
        'Ĳ' | 'ĳ' => Some("ij"),
        'ı' => Some("i"),
        'Ŋ' | 'ŋ' => Some("ng"),
        'Ə' | 'ə' | 'Ɛ' | 'ɛ' => Some("e"),
        'Ƒ' | 'ƒ' => Some("f"),
        'ĸ' => Some("k"),
        // Serbo-Croatian digraph code points (both the NFKD-decomposable and
        // the titlecase/lowercase forms).
        'Ǆ' | 'ǅ' | 'ǆ' | 'Ǳ' | 'ǲ' | 'ǳ' => Some("dz"),
        'Ǉ' | 'ǈ' | 'ǉ' => Some("lj"),
        'Ǌ' | 'ǋ' | 'ǌ' => Some("nj"),
        // Typographic ligatures (Alphabetic Presentation Forms).
        'ﬀ' => Some("ff"),
        'ﬁ' => Some("fi"),
        'ﬂ' => Some("fl"),
        'ﬃ' => Some("ffi"),
        'ﬄ' => Some("ffl"),
        'ﬅ' | 'ﬆ' => Some("st"),
        // Letterlike symbols with a multi-letter compatibility decomposition.
        '™' => Some("tm"),
        '№' => Some("no"),
        _ => None,
    }
}

/// Greek letters to their common Latin romanisation (lowercase input only —
/// callers fold case first). Accented Greek decomposes via NFD to one of
/// these base letters plus combining marks.
fn greek(c: char) -> Option<&'static str> {
    match c {
        'α' => Some("a"),
        'β' => Some("b"),
        'γ' => Some("g"),
        'δ' => Some("d"),
        'ε' => Some("e"),
        'ζ' => Some("z"),
        'η' | 'ι' => Some("i"),
        'θ' => Some("th"),
        'κ' => Some("k"),
        'λ' => Some("l"),
        'μ' => Some("m"),
        'ν' => Some("n"),
        'ξ' => Some("x"),
        'ο' | 'ω' => Some("o"),
        'π' => Some("p"),
        'ρ' => Some("r"),
        'σ' | 'ς' => Some("s"),
        'τ' => Some("t"),
        'υ' => Some("y"),
        'φ' => Some("f"),
        'χ' => Some("ch"),
        'ψ' => Some("ps"),
        _ => None,
    }
}

/// Cyrillic letters (Russian plus common Ukrainian/Belarusian extras) to a
/// simplified English romanisation (lowercase input only — callers fold case
/// first). The hard and soft signs (`ъ`, `ь`) carry no sound and are dropped.
fn cyrillic(c: char) -> Option<&'static str> {
    match c {
        'а' => Some("a"),
        'б' => Some("b"),
        'в' => Some("v"),
        'г' | 'ґ' => Some("g"),
        'д' => Some("d"),
        'е' | 'ё' | 'э' => Some("e"),
        'є' => Some("ye"),
        'ж' => Some("zh"),
        'з' => Some("z"),
        'и' | 'й' | 'і' | 'ї' => Some("i"),
        'к' => Some("k"),
        'л' => Some("l"),
        'м' => Some("m"),
        'н' => Some("n"),
        'о' => Some("o"),
        'п' => Some("p"),
        'р' => Some("r"),
        'с' => Some("s"),
        'т' => Some("t"),
        'у' | 'ў' => Some("u"),
        'ф' => Some("f"),
        'х' => Some("kh"),
        'ц' => Some("ts"),
        'ч' => Some("ch"),
        'ш' => Some("sh"),
        'щ' => Some("shch"),
        'ъ' | 'ь' => Some(""),
        'ы' => Some("y"),
        'ю' => Some("yu"),
        'я' => Some("ya"),
        _ => None,
    }
}

/// First char of the Unicode lowercase mapping of `c` (identity when the
/// mapping is empty). Enough for the single-char table lookups: Greek and
/// Cyrillic capitals all lowercase to exactly one code point.
fn fold_lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Transliterate a single Unicode character to its ASCII equivalent(s).
///
/// Pipeline:
/// 1. Combining marks (e.g. U+0301 acute) are dropped (`""`).
/// 2. ASCII alphanumerics are lowercased.
/// 3. `_` is preserved.
/// 4. A special map covers non-decomposable Latin letters, digraphs,
///    typographic ligatures and letterlike symbols
///    (`Æ`, `Œ`, `ß`, `Þ`, `Ø`, `Ł`, `Đ`/`Ð`, `ﬁ`, `™`, `№`…).
/// 5. Greek and Cyrillic letters are romanised via dedicated tables
///    (case-folded first).
/// 6. Otherwise the char is NFKD-decomposed; if the base is ASCII
///    alphanumeric its lowercase form is returned (covers the entire Latin
///    Extended-A block, fullwidth forms, superscripts…); if the base is
///    Greek or Cyrillic the tables above apply (accented Greek, `Ё`…).
/// 7. Everything else (spaces, punctuation, CJK, emoji…) returns `"-"`.
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
    let folded = fold_lower(c);
    if let Some(s) = greek(folded).or_else(|| cyrillic(folded)) {
        return s;
    }

    // NFKD fallback: covers any precomposed letter whose compatibility
    // decomposition starts with a known base (À, é, Č, Ą, ș, İ, Ａ, ², ά, Й…).
    if let Some(base) = c.nfkd().find(|x| !is_combining_mark(*x)) {
        if base != c {
            if base.is_ascii_alphanumeric() {
                return ascii_alnum_to_lower(base);
            }
            let base = fold_lower(base);
            if let Some(s) = greek(base).or_else(|| cyrillic(base)) {
                return s;
            }
        }
    }
    "-"
}

// ─────────────────────────────────────────────────────────────────────────────
// Optional cleanup fixes (mojibake repair, HTML stripping)
// ─────────────────────────────────────────────────────────────────────────────

/// Optional cleanup passes applied to a name **before** the slug pipeline.
///
/// The default (`CleanupOptions::default()`) applies nothing, matching the
/// historical behaviour of `transform_filename` / `transform_dirname`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CleanupOptions {
    /// Repair mojibake: UTF-8 byte sequences that were wrongly decoded as
    /// Latin-1 / Windows-1252 (`CafÃ©` → `Café`). See [`fix_unicode`].
    pub fix_unicode: bool,
    /// Strip HTML tags and decode HTML entities (`<b>Tom &amp; Jerry</b>` →
    /// `Tom & Jerry`). See [`fix_html`].
    pub fix_html: bool,
}

impl CleanupOptions {
    /// Every cleanup fix enabled (the CLI `-A`/`--fix-all` option).
    #[must_use]
    pub fn all() -> Self {
        Self {
            fix_unicode: true,
            fix_html: true,
        }
    }

    /// Apply the enabled fixes in order: mojibake repair first (it restores
    /// the real characters HTML stripping then operates on), then HTML.
    fn apply(self, name: &str) -> String {
        let mut out = name.to_owned();
        if self.fix_unicode {
            out = fix_unicode(&out);
        }
        if self.fix_html {
            out = fix_html(&out);
        }
        out
    }
}

/// Map a char back to the single Latin-1 / Windows-1252 byte whose
/// misinterpretation produced it, or `None` when it cannot come from one.
///
/// Code points `U+0000..=U+00FF` map to themselves (Latin-1); the extra arms
/// cover the 27 printable Windows-1252 characters sitting in the
/// `0x80..=0x9F` range (`€`, `’`, `“`, `–`, `œ`…), which are what a CP1252
/// misdecoding shows instead of raw control bytes.
fn cp1252_byte(c: char) -> Option<u8> {
    if let Ok(b) = u8::try_from(u32::from(c)) {
        return Some(b);
    }
    let b = match c {
        '€' => 0x80,
        '‚' => 0x82,
        'ƒ' => 0x83,
        '„' => 0x84,
        '…' => 0x85,
        '†' => 0x86,
        '‡' => 0x87,
        'ˆ' => 0x88,
        '‰' => 0x89,
        'Š' => 0x8A,
        '‹' => 0x8B,
        'Œ' => 0x8C,
        'Ž' => 0x8E,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '˜' => 0x98,
        '™' => 0x99,
        'š' => 0x9A,
        '›' => 0x9B,
        'œ' => 0x9C,
        'ž' => 0x9E,
        'Ÿ' => 0x9F,
        _ => return None,
    };
    Some(b)
}

/// Repair mojibake: a name whose UTF-8 bytes were wrongly decoded as Latin-1
/// or Windows-1252 (`CafÃ©` → `Café`, `Tomâ€™s` → `Tom’s`).
///
/// The repair is all-or-nothing per pass: every character must map back to a
/// single Latin-1/CP1252 byte **and** the resulting byte string must be valid
/// UTF-8, otherwise the name is returned unchanged. Names that are already
/// correct are left alone (pure ASCII round-trips to itself; a lone `é` maps
/// to the invalid UTF-8 byte `0xE9`). Up to three passes handle double and
/// triple mojibake (`CafÃƒÂ©` → `CafÃ©` → `Café`).
#[must_use]
pub fn fix_unicode(name: &str) -> String {
    let mut current = name.to_owned();
    for _ in 0..3 {
        let Some(bytes) = current
            .chars()
            .map(cp1252_byte)
            .collect::<Option<Vec<u8>>>()
        else {
            break;
        };
        match String::from_utf8(bytes) {
            Ok(decoded) if decoded != current => current = decoded,
            _ => break,
        }
    }
    current
}

/// Block-level tags whose removal would otherwise weld two words together
/// (`data<br>client`). Their names are matched case-insensitively; when such a
/// tag is stripped it is replaced by a space so the slug pipeline turns it into
/// a separator. `h1`..`h6` are handled separately by [`is_separator_tag`].
const SEPARATOR_TAGS: &[&str] = &[
    "br",
    "p",
    "div",
    "li",
    "ul",
    "ol",
    "dl",
    "dt",
    "dd",
    "tr",
    "td",
    "th",
    "table",
    "thead",
    "tbody",
    "tfoot",
    "hr",
    "section",
    "article",
    "header",
    "footer",
    "nav",
    "aside",
    "main",
    "figure",
    "figcaption",
    "blockquote",
    "pre",
    "address",
    "form",
    "fieldset",
    "caption",
];

/// Does the content between `<` and `>` name a block-level tag that should be
/// replaced by a space? Leading `/` (closing tag) is ignored; the tag name is
/// read up to the first non-alphanumeric byte, then matched case-insensitively.
fn is_separator_tag(content: &str) -> bool {
    let name = content.strip_prefix('/').unwrap_or(content);
    let end = name
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(name.len());
    let name = name[..end].to_ascii_lowercase();
    // Headings `h1`..`h6`.
    if let Some(rest) = name.strip_prefix('h') {
        if rest.len() == 1 && matches!(rest.as_bytes()[0], b'1'..=b'6') {
            return true;
        }
    }
    SEPARATOR_TAGS.contains(&name.as_str())
}

/// Remove HTML/XML tags: a `<` immediately followed by an ASCII letter, `/`
/// or `!` opens a tag that ends at the next `>`. Anything else (`a < b`, an
/// unclosed `<tag`) is kept verbatim and left to the slug pipeline.
///
/// Block-level tags (`<br>`, `<p>`, `<h1>`…) are replaced by a space so the
/// words around them stay separate; inline tags (`<b>`, `<i>`…) vanish with no
/// gap (`client<b>s` → `clients`).
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('<') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        let opens_tag = matches!(
            after[1..].chars().next(),
            Some(c) if c.is_ascii_alphabetic() || c == '/' || c == '!'
        );
        if opens_tag {
            if let Some(end) = after.find('>') {
                if is_separator_tag(&after[1..end]) {
                    out.push(' ');
                }
                rest = &after[end + 1..];
                continue;
            }
        }
        out.push('<');
        rest = &after[1..];
    }
    out.push_str(rest);
    out
}

/// Decode a named HTML entity body (the part between `&` and `;`).
///
/// Covers the XML core, common typographic symbols and the Latin-1 accented
/// letters. Lookup is retried in lowercase so `&Eacute;` resolves too (the
/// slug pipeline lowercases everything anyway).
fn named_entity(body: &str) -> Option<char> {
    let c = match body {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "deg" => '°',
        "plusmn" => '±',
        "middot" => '·',
        "bull" => '•',
        "hellip" => '…',
        "ndash" => '–',
        "mdash" => '—',
        "lsquo" => '‘',
        "rsquo" => '’',
        "ldquo" => '“',
        "rdquo" => '”',
        "laquo" => '«',
        "raquo" => '»',
        "sect" => '§',
        "para" => '¶',
        "times" => '×',
        "divide" => '÷',
        "euro" => '€',
        "pound" => '£',
        "cent" => '¢',
        "yen" => '¥',
        "agrave" => 'à',
        "aacute" => 'á',
        "acirc" => 'â',
        "atilde" => 'ã',
        "auml" => 'ä',
        "aring" => 'å',
        "aelig" => 'æ',
        "ccedil" => 'ç',
        "egrave" => 'è',
        "eacute" => 'é',
        "ecirc" => 'ê',
        "euml" => 'ë',
        "igrave" => 'ì',
        "iacute" => 'í',
        "icirc" => 'î',
        "iuml" => 'ï',
        "ntilde" => 'ñ',
        "ograve" => 'ò',
        "oacute" => 'ó',
        "ocirc" => 'ô',
        "otilde" => 'õ',
        "ouml" => 'ö',
        "oslash" => 'ø',
        "oelig" => 'œ',
        "szlig" => 'ß',
        "eth" => 'ð',
        "thorn" => 'þ',
        "ugrave" => 'ù',
        "uacute" => 'ú',
        "ucirc" => 'û',
        "uuml" => 'ü',
        "yacute" => 'ý',
        "yuml" => 'ÿ',
        _ => return None,
    };
    Some(c)
}

/// Decode one entity body: numeric (`#233`, `#xE9`) or named (`eacute`).
fn decode_entity_body(body: &str) -> Option<char> {
    if let Some(num) = body.strip_prefix('#') {
        let cp = if let Some(hex) = num.strip_prefix(['x', 'X']) {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            num.parse::<u32>().ok()?
        };
        return char::from_u32(cp).filter(|c| !c.is_control());
    }
    named_entity(body).or_else(|| named_entity(&body.to_ascii_lowercase()))
}

/// Longest plausible entity body (`&frac34;` style names stay well under it).
const MAX_ENTITY_LEN: usize = 12;

/// Replace decodable HTML entities with their character; anything that does
/// not parse as an entity (`Tom & Jerry`, `&zzz;`) is kept verbatim.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        let decoded = after[1..]
            .find(';')
            .filter(|&semi| semi <= MAX_ENTITY_LEN)
            .and_then(|semi| decode_entity_body(&after[1..=semi]).map(|c| (c, semi)));
        if let Some((c, semi)) = decoded {
            out.push(c);
            rest = &after[semi + 2..];
        } else {
            out.push('&');
            rest = &after[1..];
        }
    }
    out.push_str(rest);
    out
}

/// Strip HTML tags then decode HTML entities.
///
/// Tags are removed first so `&lt;` decoding cannot create new "tags";
/// a decoded `<` or `>` simply becomes a `-` in the slug pipeline.
#[must_use]
pub fn fix_html(name: &str) -> String {
    decode_entities(&strip_tags(name))
}

// ─────────────────────────────────────────────────────────────────────────────
// String transformation pipeline
// ─────────────────────────────────────────────────────────────────────────────

/// Collapse any run of consecutive `sep` characters into a single `sep`.
fn collapse_runs(s: &str, sep: char) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_sep = false;
    for c in s.chars() {
        if c == sep {
            if !prev_sep {
                out.push(sep);
            }
            prev_sep = true;
        } else {
            prev_sep = false;
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
/// 1. Normalise the input to NFKD so accented letters split into base + marks
///    and compatibility forms (ligatures `ﬁ`, fullwidth `Ａ`, superscripts
///    `²`…) decompose to their plain equivalents. This makes the
///    transformation idempotent regardless of whether the input filename was
///    stored as NFC (`café`) or NFD (`cafe\u{0301}`).
/// 2. Transliterate every character (combining marks become empty).
/// 3. Collapse consecutive `-`.
/// 4. Remove `-` adjacent to `_` (`_-` → `_`, `-_` → `_`).
/// 5. Collapse consecutive `_` (step 4 can produce `__` from e.g. `_-_`).
/// 6. Trim leading / trailing `-` and `_`.
#[must_use]
pub fn transform_stem(stem: &str) -> String {
    let raw: String = stem.nfkd().map(transliterate_char).collect();
    let collapsed = collapse_runs(&raw, '-');
    let fixed = fix_underscore_dash(&collapsed);
    let fixed = collapse_runs(&fixed, '_');
    trim_separators(&fixed)
}

/// Known compound extensions that must be kept together, stored with their
/// leading dot and in lowercase — matched case-insensitively. Add new ones here.
const DOUBLE_EXTENSIONS: &[&str] = &[".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst"];

/// Extract a compound extension if the filename ends with one of the known
/// double extensions (case-insensitive), and return `(stem, ".compound.ext")`.
/// Falls back to the standard single-extension split otherwise.
fn split_extension(filename: &str) -> (&str, String) {
    // Match the known compound extensions case-insensitively by comparing the
    // trailing bytes directly — no lowercased copy of the whole name and no
    // per-iteration `format!`. A match implies the tail is pure ASCII, so
    // `start` is always a valid char boundary for slicing `filename`.
    let bytes = filename.as_bytes();
    for &ext in DOUBLE_EXTENSIONS {
        let Some(start) = bytes.len().checked_sub(ext.len()) else {
            continue;
        };
        if bytes[start..].eq_ignore_ascii_case(ext.as_bytes()) {
            return (&filename[..start], ext.to_owned());
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
    transform_filename_with(filename, CleanupOptions::default())
}

/// Like [`transform_filename`], with optional cleanup fixes applied to the
/// whole name (mojibake repair, HTML stripping) **before** the extension is
/// split off — an HTML tag or a mojibake sequence may contain dots that would
/// otherwise confuse the extension detection.
#[must_use]
pub fn transform_filename_with(filename: &str, opts: CleanupOptions) -> String {
    // Leave hidden files alone
    if filename.starts_with('.') {
        return filename.to_owned();
    }

    let cleaned = opts.apply(filename);
    let (stem, ext) = split_extension(&cleaned);
    let new_stem = transform_stem(stem);

    if new_stem.is_empty() {
        return format!("unnamed{ext}");
    }

    format!("{new_stem}{ext}")
}

/// Transform a **directory** name into a clean ASCII slug.
///
/// Unlike `transform_filename`, directories have no notion of an extension:
/// a dot is just a regular character, so the whole name goes through
/// `transform_stem` (e.g. `My Project.v2` → `my-project-v2`, not
/// `my-project.v2`). Hidden entries (names starting with `.`) are returned
/// unchanged. An entry that transliterates to nothing becomes `unnamed`.
#[must_use]
pub fn transform_dirname(name: &str) -> String {
    transform_dirname_with(name, CleanupOptions::default())
}

/// Like [`transform_dirname`], with optional cleanup fixes applied first
/// (mojibake repair, HTML stripping).
#[must_use]
pub fn transform_dirname_with(name: &str, opts: CleanupOptions) -> String {
    // Leave hidden directories alone
    if name.starts_with('.') {
        return name.to_owned();
    }

    let new_name = transform_stem(&opts.apply(name));
    if new_name.is_empty() {
        return "unnamed".to_owned();
    }
    new_name
}

// ─────────────────────────────────────────────────────────────────────────────
// Duplicate detection
// ─────────────────────────────────────────────────────────────────────────────

/// Outcome of comparing a rename source with an already existing destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryMatch {
    /// Both paths resolve to the very same filesystem entry (a hard link, a
    /// symlink to the other one, or a case-insensitive filesystem folding the
    /// two names together). Removing either one could destroy the last copy,
    /// so callers must treat this as a conflict, never as a duplicate.
    SameEntry,
    /// Two distinct regular files with byte-for-byte identical content.
    Identical,
    /// Two distinct regular files whose contents differ.
    Different,
    /// At least one path is not a regular file (directory, socket, fifo…):
    /// there is nothing to compare byte for byte.
    NotComparable,
}

/// Compare two existing paths to decide whether one is a strict duplicate of
/// the other.
///
/// Symlinks are followed, so what is compared is the content they resolve to.
/// Files are compared by identity first, then by size, then chunk by chunk:
/// differing files bail out early and no whole-file digest is ever computed,
/// which makes an exact answer cheaper than hashing both sides.
pub fn compare_entries(a: &Path, b: &Path) -> io::Result<EntryMatch> {
    let (meta_a, meta_b) = (fs::metadata(a)?, fs::metadata(b)?);

    if is_same_entry(&meta_a, &meta_b) {
        return Ok(EntryMatch::SameEntry);
    }
    if !meta_a.is_file() || !meta_b.is_file() {
        return Ok(EntryMatch::NotComparable);
    }
    if meta_a.len() != meta_b.len() {
        return Ok(EntryMatch::Different);
    }
    if contents_equal(a, b)? {
        Ok(EntryMatch::Identical)
    } else {
        Ok(EntryMatch::Different)
    }
}

/// Whether two metadata snapshots describe the same filesystem entry.
///
/// Unix compares the `(device, inode)` pair. Elsewhere there is no portable
/// identity to compare, so the check stays conservative and answers `false`;
/// the content comparison then decides.
#[cfg(unix)]
fn is_same_entry(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    a.dev() == b.dev() && a.ino() == b.ino()
}

#[cfg(not(unix))]
fn is_same_entry(_a: &fs::Metadata, _b: &fs::Metadata) -> bool {
    false
}

/// Byte-for-byte comparison of two files, streamed through the reader buffers
/// so neither file is ever fully loaded in memory.
fn contents_equal(a: &Path, b: &Path) -> io::Result<bool> {
    let mut reader_a = BufReader::new(fs::File::open(a)?);
    let mut reader_b = BufReader::new(fs::File::open(b)?);

    loop {
        let buf_a = reader_a.fill_buf()?;
        let buf_b = reader_b.fill_buf()?;

        // End of file on either side: equal only if both ended together.
        if buf_a.is_empty() || buf_b.is_empty() {
            return Ok(buf_a.is_empty() && buf_b.is_empty());
        }

        // The two buffers rarely hold the same amount of data; compare the
        // common prefix and consume exactly that much on both sides.
        let len = buf_a.len().min(buf_b.len());
        if buf_a[..len] != buf_b[..len] {
            return Ok(false);
        }
        reader_a.consume(len);
        reader_b.consume(len);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Filesystem operations
// ─────────────────────────────────────────────────────────────────────────────

/// Controls which filesystem entries are processed by `plan_rename`.
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

/// Outcome of planning a rename for a single explicit entry.
///
/// Unlike `plan_rename`, this distinguishes an entry that is *matched but
/// already clean* (a no-op worth reporting as skipped) from one that is *not a
/// rename candidate at all* (filtered out by the type flag, or an invalid
/// UTF-8 name). The CLI needs that distinction to report the `[X]` lines and
/// the "matched" count correctly.
#[derive(Debug, Clone)]
pub enum RenamePlan {
    /// The entry needs renaming.
    Rename(RenameOp),
    /// The entry matched the target filter but is already clean: nothing to do.
    AlreadyClean,
    /// The entry is not a rename candidate: it does not match the target
    /// filter (e.g. a directory under `FilesOnly`), or its name is not valid
    /// UTF-8.
    Excluded,
}

/// Classify a single explicit entry — the entry **itself**, not its contents.
///
/// Used for paths passed directly on the command line (the `rename`-like
/// mode). `is_file` / `is_dir` follow symlinks. When a rename is needed the
/// destination keeps the entry's parent directory and only swaps the basename.
#[must_use]
pub fn plan_entry(path: &Path, target: RenameTarget) -> RenamePlan {
    plan_entry_with(path, target, CleanupOptions::default())
}

/// Like [`plan_entry`], with optional cleanup fixes applied to the name
/// (mojibake repair, HTML stripping) before the slug pipeline.
#[must_use]
pub fn plan_entry_with(path: &Path, target: RenameTarget, opts: CleanupOptions) -> RenamePlan {
    let is_file = path.is_file();
    let is_dir = path.is_dir();

    let include = match target {
        RenameTarget::All => is_file || is_dir,
        RenameTarget::FilesOnly => is_file,
        RenameTarget::DirsOnly => is_dir,
    };
    if !include {
        return RenamePlan::Excluded;
    }

    let Some(original) = path.file_name().and_then(|n| n.to_str()) else {
        return RenamePlan::Excluded;
    };

    // Directories have no extension (a dot is a plain separator), so route them
    // through transform_dirname; files through the extension-aware transform.
    let renamed = if is_dir {
        transform_dirname_with(original, opts)
    } else {
        transform_filename_with(original, opts)
    };

    if renamed == original {
        return RenamePlan::AlreadyClean; // already clean or hidden file
    }

    let to = path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&renamed);
    RenamePlan::Rename(RenameOp {
        from: path.to_path_buf(),
        to,
    })
}

/// Compute the rename for a single explicit entry, returning `None` when there
/// is nothing to do (already clean, filtered out, or invalid UTF-8).
///
/// Thin wrapper over `plan_entry` for callers that only care whether a rename
/// is needed.
#[must_use]
pub fn plan_rename(path: &Path, target: RenameTarget) -> Option<RenameOp> {
    match plan_entry(path, target) {
        RenamePlan::Rename(op) => Some(op),
        RenamePlan::AlreadyClean | RenamePlan::Excluded => None,
    }
}
