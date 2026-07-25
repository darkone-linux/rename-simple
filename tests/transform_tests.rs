use rename_files::{
    fix_html, fix_unicode, transform_dirname, transform_dirname_with, transform_filename,
    transform_filename_with, transform_stem, transliterate_char, CleanupOptions,
};

// ─────────────────────────────────────────────────────────────────────────────
// transliterate_char
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod transliterate_char_tests {
    use super::*;

    #[test]
    fn ascii_lowercase_letters_are_preserved() {
        for c in 'a'..='z' {
            assert_eq!(
                transliterate_char(c),
                c.to_string().as_str(),
                "expected '{c}' to stay '{c}'"
            );
        }
    }

    #[test]
    fn ascii_uppercase_letters_are_lowercased() {
        assert_eq!(transliterate_char('A'), "a");
        assert_eq!(transliterate_char('Z'), "z");
        assert_eq!(transliterate_char('M'), "m");
    }

    #[test]
    fn digits_are_preserved() {
        for c in '0'..='9' {
            assert_eq!(
                transliterate_char(c),
                c.to_string().as_str(),
                "expected digit '{c}' to stay '{c}'"
            );
        }
    }

    #[test]
    fn underscore_is_preserved() {
        assert_eq!(transliterate_char('_'), "_");
    }

    // — A variants ────────────────────────────────────────────────────────────
    #[test]
    fn a_variants() {
        for c in ['à', 'á', 'â', 'ã', 'ä', 'å', 'À', 'Á', 'Â', 'Ã', 'Ä', 'Å'] {
            assert_eq!(transliterate_char(c), "a", "failed for '{c}'");
        }
    }

    #[test]
    fn ae_ligature() {
        assert_eq!(transliterate_char('æ'), "ae");
        assert_eq!(transliterate_char('Æ'), "ae");
    }

    // — C variants ────────────────────────────────────────────────────────────
    #[test]
    fn c_cedilla() {
        assert_eq!(transliterate_char('ç'), "c");
        assert_eq!(transliterate_char('Ç'), "c");
    }

    // — D variants ────────────────────────────────────────────────────────────
    #[test]
    fn eth() {
        assert_eq!(transliterate_char('ð'), "d");
        assert_eq!(transliterate_char('Ð'), "d");
    }

    // — E variants ────────────────────────────────────────────────────────────
    #[test]
    fn e_variants() {
        for c in ['è', 'é', 'ê', 'ë', 'È', 'É', 'Ê', 'Ë'] {
            assert_eq!(transliterate_char(c), "e", "failed for '{c}'");
        }
    }

    // — I variants ────────────────────────────────────────────────────────────
    #[test]
    fn i_variants() {
        for c in ['ì', 'í', 'î', 'ï', 'Ì', 'Í', 'Î', 'Ï'] {
            assert_eq!(transliterate_char(c), "i", "failed for '{c}'");
        }
    }

    // — N variants ────────────────────────────────────────────────────────────
    #[test]
    fn n_tilde() {
        assert_eq!(transliterate_char('ñ'), "n");
        assert_eq!(transliterate_char('Ñ'), "n");
    }

    // — O variants ────────────────────────────────────────────────────────────
    #[test]
    fn o_variants() {
        for c in ['ò', 'ó', 'ô', 'õ', 'ö', 'ø', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', 'Ø'] {
            assert_eq!(transliterate_char(c), "o", "failed for '{c}'");
        }
    }

    #[test]
    fn oe_ligature() {
        assert_eq!(transliterate_char('œ'), "oe");
        assert_eq!(transliterate_char('Œ'), "oe");
    }

    // — S variants ────────────────────────────────────────────────────────────
    #[test]
    fn sharp_s() {
        assert_eq!(transliterate_char('ß'), "ss");
    }

    // — T variants ────────────────────────────────────────────────────────────
    #[test]
    fn thorn() {
        assert_eq!(transliterate_char('þ'), "th");
        assert_eq!(transliterate_char('Þ'), "th");
    }

    // — U variants ────────────────────────────────────────────────────────────
    #[test]
    fn u_variants() {
        for c in ['ù', 'ú', 'û', 'ü', 'Ù', 'Ú', 'Û', 'Ü'] {
            assert_eq!(transliterate_char(c), "u", "failed for '{c}'");
        }
    }

    // — Y variants ────────────────────────────────────────────────────────────
    #[test]
    fn y_variants() {
        for c in ['ý', 'ÿ', 'Ý', 'Ÿ'] {
            assert_eq!(transliterate_char(c), "y", "failed for '{c}'");
        }
    }

    // — Z variants ────────────────────────────────────────────────────────────
    #[test]
    fn z_variants() {
        for c in ['ź', 'ż', 'ž', 'Ź', 'Ż', 'Ž'] {
            assert_eq!(transliterate_char(c), "z", "failed for '{c}'");
        }
    }

    // — Latin Extended-A (covered automatically via NFD decomposition) ────────
    #[test]
    fn latin_extended_a_macron_breve_ogonek_caron() {
        // Macron, breve, ogonek, caron — all decompose to base + combining mark
        for c in ['Ā', 'Ă', 'Ą', 'ā', 'ă', 'ą'] {
            assert_eq!(transliterate_char(c), "a", "failed for '{c}'");
        }
        for c in ['Ē', 'Ĕ', 'Ė', 'Ę', 'Ě', 'ē', 'ĕ', 'ė', 'ę', 'ě'] {
            assert_eq!(transliterate_char(c), "e", "failed for '{c}'");
        }
        for c in ['Č', 'Ć', 'Ċ', 'Ĉ', 'č', 'ć', 'ċ', 'ĉ'] {
            assert_eq!(transliterate_char(c), "c", "failed for '{c}'");
        }
        for c in ['Š', 'Ś', 'Ŝ', 'Ş', 'š', 'ś', 'ŝ', 'ş'] {
            assert_eq!(transliterate_char(c), "s", "failed for '{c}'");
        }
        for c in ['Ř', 'Ŕ', 'Ŗ', 'ř', 'ŕ', 'ŗ'] {
            assert_eq!(transliterate_char(c), "r", "failed for '{c}'");
        }
    }

    // — Latin Extended-A non-decomposable (covered via special_latin map) ─────
    #[test]
    fn latin_extended_l_with_stroke() {
        assert_eq!(transliterate_char('Ł'), "l");
        assert_eq!(transliterate_char('ł'), "l");
    }

    #[test]
    fn latin_extended_d_with_stroke() {
        // Croatian / Vietnamese
        assert_eq!(transliterate_char('Đ'), "d");
        assert_eq!(transliterate_char('đ'), "d");
    }

    #[test]
    fn latin_extended_h_with_stroke() {
        // Maltese
        assert_eq!(transliterate_char('Ħ'), "h");
        assert_eq!(transliterate_char('ħ'), "h");
    }

    #[test]
    fn latin_extended_t_with_stroke() {
        assert_eq!(transliterate_char('Ŧ'), "t");
        assert_eq!(transliterate_char('ŧ'), "t");
    }

    #[test]
    fn latin_extended_ij_ligature() {
        // Dutch IJ
        assert_eq!(transliterate_char('Ĳ'), "ij");
        assert_eq!(transliterate_char('ĳ'), "ij");
    }

    #[test]
    fn turkish_dotted_and_dotless_i() {
        // I with dot above (capital): NFD → I + combining dot above → "i"
        assert_eq!(transliterate_char('İ'), "i");
        // Dotless small i: non-decomposable, handled by special_latin
        assert_eq!(transliterate_char('ı'), "i");
    }

    #[test]
    fn romanian_letters_with_comma_below() {
        // Modern Romanian uses comma-below (U+0218..U+021B), older fonts
        // sometimes use cedilla (Ş Ţ); both must transliterate identically.
        assert_eq!(transliterate_char('Ș'), "s");
        assert_eq!(transliterate_char('ș'), "s");
        assert_eq!(transliterate_char('Ț'), "t");
        assert_eq!(transliterate_char('ț'), "t");
        assert_eq!(transliterate_char('Ş'), "s");
        assert_eq!(transliterate_char('ş'), "s");
        assert_eq!(transliterate_char('Ţ'), "t");
        assert_eq!(transliterate_char('ţ'), "t");
    }

    // — Combining marks are dropped ──────────────────────────────────────────
    #[test]
    fn combining_marks_are_dropped() {
        // Combining acute, grave, circumflex, tilde, diaeresis
        for c in ['\u{0301}', '\u{0300}', '\u{0302}', '\u{0303}', '\u{0308}'] {
            assert_eq!(transliterate_char(c), "", "failed for U+{:04X}", c as u32);
        }
        // Combining cedilla, ogonek, caron, macron, breve
        for c in ['\u{0327}', '\u{0328}', '\u{030C}', '\u{0304}', '\u{0306}'] {
            assert_eq!(transliterate_char(c), "", "failed for U+{:04X}", c as u32);
        }
    }

    // — Non-Latin scripts fall back to dash ──────────────────────────────────
    #[test]
    fn cjk_becomes_dash() {
        for c in ['你', '好', '中', '日', '本', '한', '글'] {
            assert_eq!(transliterate_char(c), "-", "failed for '{c}'");
        }
    }

    #[test]
    fn emoji_becomes_dash() {
        for c in ['🦀', '📦', '✓', '★'] {
            assert_eq!(transliterate_char(c), "-", "failed for '{c}'");
        }
    }

    // — Separators ────────────────────────────────────────────────────────────
    #[test]
    fn space_becomes_dash() {
        assert_eq!(transliterate_char(' '), "-");
    }

    #[test]
    fn punctuation_becomes_dash() {
        for c in [
            '.', ',', ';', '!', '?', ':', '(', ')', '[', ']', '\'', '"', '/', '\\',
        ] {
            assert_eq!(transliterate_char(c), "-", "failed for '{c}'");
        }
    }

    #[test]
    fn symbols_become_dash() {
        for c in ['@', '#', '$', '%', '&', '*', '+', '=', '~', '^', '`'] {
            assert_eq!(transliterate_char(c), "-", "failed for '{c}'");
        }
    }

    // — Extra non-decomposable Latin letters and digraphs ─────────────────────
    #[test]
    fn capital_sharp_s() {
        assert_eq!(transliterate_char('ẞ'), "ss");
    }

    #[test]
    fn eng_letter() {
        // Sami / African languages
        assert_eq!(transliterate_char('Ŋ'), "ng");
        assert_eq!(transliterate_char('ŋ'), "ng");
    }

    #[test]
    fn schwa_and_open_vowels() {
        // Azerbaijani schwa, African open e / open o
        for c in ['Ə', 'ə', 'Ɛ', 'ɛ'] {
            assert_eq!(transliterate_char(c), "e", "failed for '{c}'");
        }
        for c in ['Ɔ', 'ɔ'] {
            assert_eq!(transliterate_char(c), "o", "failed for '{c}'");
        }
    }

    #[test]
    fn f_with_hook_and_kra() {
        assert_eq!(transliterate_char('ƒ'), "f");
        assert_eq!(transliterate_char('ĸ'), "k");
    }

    #[test]
    fn serbo_croatian_digraphs() {
        for c in ['Ǆ', 'ǅ', 'ǆ', 'Ǳ', 'ǲ', 'ǳ'] {
            assert_eq!(transliterate_char(c), "dz", "failed for '{c}'");
        }
        for c in ['Ǉ', 'ǈ', 'ǉ'] {
            assert_eq!(transliterate_char(c), "lj", "failed for '{c}'");
        }
        for c in ['Ǌ', 'ǋ', 'ǌ'] {
            assert_eq!(transliterate_char(c), "nj", "failed for '{c}'");
        }
    }

    // — Compatibility (NFKD) forms ────────────────────────────────────────────
    #[test]
    fn typographic_ligatures_expand() {
        assert_eq!(transliterate_char('ﬀ'), "ff");
        assert_eq!(transliterate_char('ﬁ'), "fi");
        assert_eq!(transliterate_char('ﬂ'), "fl");
        assert_eq!(transliterate_char('ﬃ'), "ffi");
        assert_eq!(transliterate_char('ﬄ'), "ffl");
        assert_eq!(transliterate_char('ﬆ'), "st");
    }

    #[test]
    fn letterlike_symbols_expand() {
        assert_eq!(transliterate_char('™'), "tm");
        assert_eq!(transliterate_char('№'), "no");
    }

    #[test]
    fn fullwidth_forms_map_to_ascii() {
        assert_eq!(transliterate_char('Ａ'), "a");
        assert_eq!(transliterate_char('ｚ'), "z");
        assert_eq!(transliterate_char('５'), "5");
    }

    #[test]
    fn superscripts_and_subscripts_map_to_digits() {
        assert_eq!(transliterate_char('²'), "2");
        assert_eq!(transliterate_char('³'), "3");
        assert_eq!(transliterate_char('₄'), "4");
    }

    #[test]
    fn long_s_maps_to_s() {
        // U+017F LATIN SMALL LETTER LONG S has a compat decomposition to 's'
        assert_eq!(transliterate_char('ſ'), "s");
    }

    #[test]
    fn micro_sign_maps_like_greek_mu() {
        // U+00B5 MICRO SIGN lowercases to Greek mu
        assert_eq!(transliterate_char('µ'), "m");
    }

    // — Greek ─────────────────────────────────────────────────────────────────
    #[test]
    fn greek_letters_romanise() {
        assert_eq!(transliterate_char('α'), "a");
        assert_eq!(transliterate_char('Ω'), "o");
        assert_eq!(transliterate_char('θ'), "th");
        assert_eq!(transliterate_char('χ'), "ch");
        assert_eq!(transliterate_char('ψ'), "ps");
        assert_eq!(transliterate_char('σ'), "s");
        assert_eq!(transliterate_char('ς'), "s"); // final sigma
    }

    #[test]
    fn accented_greek_uses_base_letter() {
        assert_eq!(transliterate_char('ά'), "a");
        assert_eq!(transliterate_char('Έ'), "e");
        assert_eq!(transliterate_char('ώ'), "o");
    }

    // — Cyrillic ──────────────────────────────────────────────────────────────
    #[test]
    fn cyrillic_letters_romanise() {
        assert_eq!(transliterate_char('ж'), "zh");
        assert_eq!(transliterate_char('Х'), "kh");
        assert_eq!(transliterate_char('ц'), "ts");
        assert_eq!(transliterate_char('Ч'), "ch");
        assert_eq!(transliterate_char('ш'), "sh");
        assert_eq!(transliterate_char('щ'), "shch");
        assert_eq!(transliterate_char('ю'), "yu");
        assert_eq!(transliterate_char('Я'), "ya");
    }

    #[test]
    fn cyrillic_signs_are_dropped() {
        assert_eq!(transliterate_char('ъ'), "");
        assert_eq!(transliterate_char('ь'), "");
    }

    #[test]
    fn cyrillic_yo_decomposes_to_e() {
        // Ё/ё canonically decompose to Е/е + diaeresis
        assert_eq!(transliterate_char('Ё'), "e");
        assert_eq!(transliterate_char('ё'), "e");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// transform_stem
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod transform_stem_tests {
    use super::*;

    // — Basic normalisation ───────────────────────────────────────────────────
    #[test]
    fn already_clean_stem_is_unchanged() {
        assert_eq!(transform_stem("hello-world"), "hello-world");
    }

    #[test]
    fn uppercase_is_lowercased() {
        assert_eq!(transform_stem("HelloWorld"), "helloworld");
        assert_eq!(transform_stem("SCREAMING"), "screaming");
    }

    #[test]
    fn spaces_become_dashes() {
        assert_eq!(transform_stem("hello world"), "hello-world");
    }

    #[test]
    fn consecutive_spaces_collapse_to_single_dash() {
        assert_eq!(transform_stem("hello   world"), "hello-world");
    }

    #[test]
    fn mixed_separators_collapse() {
        assert_eq!(transform_stem("hello - world"), "hello-world");
        assert_eq!(transform_stem("a  --  b"), "a-b");
    }

    // — Accented characters ───────────────────────────────────────────────────
    #[test]
    fn accented_chars_are_transliterated() {
        assert_eq!(transform_stem("chaîne"), "chaine");
        assert_eq!(transform_stem("café"), "cafe");
        assert_eq!(transform_stem("naïve"), "naive");
        assert_eq!(transform_stem("élève"), "eleve");
        assert_eq!(transform_stem("cœur"), "coeur");
        assert_eq!(transform_stem("façade"), "facade");
    }

    #[test]
    fn uppercase_accented_chars_are_transliterated_and_lowercased() {
        assert_eq!(transform_stem("CHÂTEAU"), "chateau");
        assert_eq!(transform_stem("ÉLÈVE"), "eleve");
    }

    // — Underscore rules ──────────────────────────────────────────────────────
    #[test]
    fn underscore_is_preserved() {
        assert_eq!(transform_stem("foo_bar"), "foo_bar");
        assert_eq!(transform_stem("01_intro"), "01_intro");
    }

    #[test]
    fn underscore_dash_becomes_underscore() {
        assert_eq!(transform_stem("foo_-bar"), "foo_bar");
        assert_eq!(transform_stem("foo-_bar"), "foo_bar");
    }

    #[test]
    fn underscore_multiple_dashes_becomes_underscore() {
        assert_eq!(transform_stem("foo_--bar"), "foo_bar");
        assert_eq!(transform_stem("foo--_bar"), "foo_bar");
    }

    #[test]
    fn underscore_surrounded_by_spaces_is_cleaned() {
        assert_eq!(transform_stem("foo _ bar"), "foo_bar");
        assert_eq!(transform_stem("01_  title"), "01_title");
    }

    #[test]
    fn chained_underscore_dash_patterns() {
        // "_-_-" should resolve cleanly
        assert_eq!(transform_stem("a_-_-b"), "a_b");
    }

    // — Leading / trailing trimming ───────────────────────────────────────────
    #[test]
    fn leading_spaces_are_trimmed() {
        assert_eq!(transform_stem("   hello"), "hello");
    }

    #[test]
    fn trailing_spaces_are_trimmed() {
        assert_eq!(transform_stem("hello   "), "hello");
    }

    #[test]
    fn leading_dashes_are_trimmed() {
        assert_eq!(transform_stem("---hello"), "hello");
    }

    #[test]
    fn trailing_dashes_are_trimmed() {
        assert_eq!(transform_stem("hello---"), "hello");
    }

    #[test]
    fn leading_underscores_are_trimmed() {
        assert_eq!(transform_stem("__hello"), "hello");
    }

    #[test]
    fn trailing_underscores_are_trimmed() {
        assert_eq!(transform_stem("hello__"), "hello");
    }

    #[test]
    fn leading_and_trailing_trimmed_together() {
        assert_eq!(transform_stem("  _-  hello world  _  "), "hello-world");
    }

    // — The specification example ─────────────────────────────────────────────
    #[test]
    fn spec_example_stem() {
        // "   01_ Cette     chaîne de      CARACtères" → "01_cette-chaine-de-caracteres"
        assert_eq!(
            transform_stem("   01_ Cette     chaîne de      CARACtères"),
            "01_cette-chaine-de-caracteres"
        );
    }

    // — Edge cases ────────────────────────────────────────────────────────────
    #[test]
    fn empty_stem_returns_empty() {
        assert_eq!(transform_stem(""), "");
    }

    #[test]
    fn only_separators_returns_empty() {
        assert_eq!(transform_stem("   ---   "), "");
        assert_eq!(transform_stem("___"), "");
    }

    #[test]
    fn numbers_only() {
        assert_eq!(transform_stem("2024"), "2024");
        assert_eq!(transform_stem("01 02 03"), "01-02-03");
    }

    #[test]
    fn ligatures_expand_correctly() {
        assert_eq!(transform_stem("æther"), "aether");
        assert_eq!(transform_stem("Œuvre"), "oeuvre");
        assert_eq!(transform_stem("straße"), "strasse");
    }

    // — Latin-Extended scripts (whole-word integration tests) ────────────────
    #[test]
    fn polish_words() {
        assert_eq!(transform_stem("Łódź"), "lodz");
        assert_eq!(transform_stem("Żółw"), "zolw");
        assert_eq!(transform_stem("Pięć Złotych"), "piec-zlotych");
    }

    #[test]
    fn czech_words() {
        assert_eq!(transform_stem("Čeština"), "cestina");
        assert_eq!(transform_stem("Příliš žluťoučký"), "prilis-zlutoucky");
    }

    #[test]
    fn croatian_words() {
        assert_eq!(transform_stem("Đak"), "dak");
        assert_eq!(transform_stem("Šljiva čokolada"), "sljiva-cokolada");
    }

    #[test]
    fn romanian_words() {
        // Both comma-below (modern) and cedilla (legacy) Ş/Ţ
        assert_eq!(transform_stem("Țuică"), "tuica");
        assert_eq!(transform_stem("România"), "romania");
        assert_eq!(transform_stem("Ţuică"), "tuica");
    }

    #[test]
    fn turkish_words() {
        assert_eq!(transform_stem("İstanbul"), "istanbul");
        assert_eq!(transform_stem("ışık"), "isik");
        assert_eq!(transform_stem("Türkçe"), "turkce");
    }

    #[test]
    fn vietnamese_d_with_stroke() {
        assert_eq!(transform_stem("Đà Nẵng"), "da-nang");
    }

    #[test]
    fn dutch_ij_ligature_in_word() {
        assert_eq!(transform_stem("Ĳsselmeer"), "ijsselmeer");
    }

    #[test]
    fn icelandic_eth_and_thorn() {
        assert_eq!(transform_stem("Þjóðverji"), "thjodverji");
    }

    // — NFD-decomposed input (idempotence) ───────────────────────────────────
    #[test]
    fn nfd_decomposed_input_is_handled() {
        // "café" written as 'c','a','f','e' + combining acute
        let nfd = "cafe\u{0301}";
        assert_eq!(transform_stem(nfd), "cafe");
        // NFC (precomposed) and NFD (decomposed) inputs must collapse to the
        // same slug — this is the property that makes filenames stable across
        // filesystems (HFS+/APFS often store names in NFD, ext4 in NFC).
        assert_eq!(transform_stem("café"), transform_stem(nfd));
    }

    #[test]
    fn nfd_decomposed_polish_input() {
        // "Łódź" precomposed vs Ł + o + combining acute + d + z + combining acute
        let nfd = "Ło\u{0301}dz\u{0301}";
        assert_eq!(transform_stem(nfd), "lodz");
        assert_eq!(transform_stem("Łódź"), transform_stem(nfd));
    }

    // — Unsupported scripts fall back to dashes ──────────────────────────────
    #[test]
    fn cjk_words_collapse_to_empty() {
        // Each CJK char becomes "-", consecutive collapse, trim → ""
        assert_eq!(transform_stem("你好"), "");
        assert_eq!(transform_stem("中文"), "");
    }

    #[test]
    fn mixed_latin_and_cjk_keeps_latin() {
        assert_eq!(transform_stem("hello 你好 world"), "hello-world");
    }

    // — Greek and Cyrillic words ──────────────────────────────────────────────
    #[test]
    fn greek_words() {
        assert_eq!(transform_stem("Ελληνικά"), "ellinika");
        assert_eq!(transform_stem("θάλασσα"), "thalassa");
        assert_eq!(transform_stem("Φιλοσοφία"), "filosofia");
    }

    #[test]
    fn russian_words() {
        assert_eq!(transform_stem("Москва"), "moskva");
        assert_eq!(transform_stem("Хорошо"), "khorosho");
        assert_eq!(transform_stem("Объект"), "obekt");
        assert_eq!(transform_stem("Ёлка"), "elka");
    }

    #[test]
    fn ukrainian_words() {
        assert_eq!(transform_stem("Київ"), "kiiv");
        assert_eq!(transform_stem("Ґанок"), "ganok");
    }

    // — Compatibility (NFKD) forms in words ───────────────────────────────────
    #[test]
    fn ligatures_in_words_expand() {
        assert_eq!(transform_stem("ﬁle ﬂow"), "file-flow");
        assert_eq!(transform_stem("diﬃcult"), "difficult");
    }

    #[test]
    fn fullwidth_words_map_to_ascii() {
        assert_eq!(transform_stem("Ｆｉｌｅ０１"), "file01");
    }

    #[test]
    fn superscripts_in_words() {
        assert_eq!(transform_stem("x² plus y³"), "x2-plus-y3");
    }

    #[test]
    fn trademark_and_numero_in_words() {
        assert_eq!(transform_stem("Brand™"), "brandtm");
        assert_eq!(transform_stem("№ 5"), "no-5");
    }

    #[test]
    fn vulgar_fraction_splits_on_fraction_slash() {
        // NFKD(½) = "1⁄2"; the fraction slash becomes a dash
        assert_eq!(transform_stem("recette ½ sucre"), "recette-1-2-sucre");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// transform_filename
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod transform_filename_tests {
    use super::*;

    // — Specification example ─────────────────────────────────────────────────
    #[test]
    fn spec_example_full_filename() {
        assert_eq!(
            transform_filename("   01_ Cette     chaîne de      CARACtères.pdf"),
            "01_cette-chaine-de-caracteres.pdf"
        );
    }

    // — Extension handling ────────────────────────────────────────────────────
    #[test]
    fn extension_is_lowercased() {
        assert_eq!(transform_filename("document.PDF"), "document.pdf");
        assert_eq!(transform_filename("image.JPEG"), "image.jpeg");
    }

    #[test]
    fn double_extension_tar_gz_is_kept_together() {
        assert_eq!(transform_filename("archive.TAR.GZ"), "archive.tar.gz"); // known -> double ext
        assert_eq!(transform_filename("archive.TOR.GZ"), "archive-tor.gz"); // unknown -> simple ext
        assert_eq!(
            transform_filename("Mon Archive.tar.gz"),
            "mon-archive.tar.gz"
        );
        assert_eq!(transform_filename("backup.TAR.BZ2"), "backup.tar.bz2");
        assert_eq!(transform_filename("backup.TAR.XZ"), "backup.tar.xz");
        assert_eq!(transform_filename("backup.TAR.ZST"), "backup.tar.zst");
    }

    #[test]
    fn no_extension_files_are_handled() {
        assert_eq!(transform_filename("README"), "readme");
        assert_eq!(transform_filename("Mon Fichier"), "mon-fichier");
    }

    #[test]
    fn extension_preserved_on_clean_name() {
        assert_eq!(transform_filename("notes.txt"), "notes.txt");
        assert_eq!(transform_filename("data.csv"), "data.csv");
        assert_eq!(transform_filename("report.docx"), "report.docx");
    }

    // — Accented filenames ────────────────────────────────────────────────────
    #[test]
    fn accented_filename_with_extension() {
        assert_eq!(
            transform_filename("Réunion d'équipe.docx"),
            "reunion-d-equipe.docx"
        );
        assert_eq!(transform_filename("procès-verbal.pdf"), "proces-verbal.pdf");
        assert_eq!(transform_filename("Données 2024.xlsx"), "donnees-2024.xlsx");
    }

    // — Separators and underscores in filenames ───────────────────────────────
    #[test]
    fn numbered_prefix_preserved() {
        assert_eq!(
            transform_filename("01_introduction.md"),
            "01_introduction.md"
        );
        assert_eq!(transform_filename("02_  Résumé.md"), "02_resume.md");
    }

    #[test]
    fn spaces_around_underscore_in_filename() {
        assert_eq!(
            transform_filename("01_ Titre du chapitre.txt"),
            "01_titre-du-chapitre.txt"
        );
    }

    // — Hidden files ──────────────────────────────────────────────────────────
    #[test]
    fn hidden_files_are_not_modified() {
        assert_eq!(transform_filename(".gitignore"), ".gitignore");
        assert_eq!(transform_filename(".hidden file"), ".hidden file");
        assert_eq!(transform_filename(".DS_Store"), ".DS_Store");
    }

    // — Edge cases ────────────────────────────────────────────────────────────
    #[test]
    fn empty_stem_after_transform_returns_unnamed() {
        assert_eq!(transform_filename("!!!(((---)))!!.txt"), "unnamed.txt");
        assert_eq!(transform_filename("???.pdf"), "unnamed.pdf");
    }

    #[test]
    fn filename_with_only_extension_separators() {
        assert_eq!(transform_filename("   .txt"), "unnamed.txt");
    }

    #[test]
    fn filename_with_numbers_and_caps() {
        assert_eq!(
            transform_filename("IMG_2024_VACANCES ÉTÉ.jpg"),
            "img_2024_vacances-ete.jpg"
        );
    }

    #[test]
    fn filename_no_dash_before_extension() {
        // trailing separators in stem must be stripped before adding extension
        assert_eq!(transform_filename("hello---.txt"), "hello.txt");
        assert_eq!(transform_filename("hello___.txt"), "hello.txt");
    }

    // — Multi-dot and compound-ext edge cases ────────────────────────────────
    #[test]
    fn multiple_dots_in_stem_become_dashes() {
        assert_eq!(transform_filename("v1.2.3.txt"), "v1-2-3.txt");
        assert_eq!(
            transform_filename("backup.2024.01.31.tar.gz"),
            "backup-2024-01-31.tar.gz"
        );
    }

    #[test]
    fn compound_ext_without_dot_prefix_treated_as_simple() {
        // "tar.gz" with no preceding stem name doesn't match `.tar.gz` (no leading dot).
        // It is parsed as stem="tar", ext=".gz".
        assert_eq!(transform_filename("tar.gz"), "tar.gz");
    }

    #[test]
    fn very_long_stem_is_preserved() {
        let stem: String = "a".repeat(200);
        let input = format!("{stem}.txt");
        assert_eq!(transform_filename(&input), input);
    }

    #[test]
    fn zero_width_space_becomes_dash() {
        // U+200B (ZWSP) is not a combining mark and has no NFD decomposition,
        // so it falls back to the dash separator.
        assert_eq!(transform_filename("abc\u{200B}def.txt"), "abc-def.txt");
    }

    // —────────────── Extension validity checks (≤ 10 ASCII alnum) ──────────────
    #[test]
    fn extension_with_non_ascii_is_absorbed_into_stem() {
        // "tét" contains é which is not ASCII alphanumeric → absorbed
        assert_eq!(transform_filename("à faire .tét"), "a-faire-tet");
    }

    #[test]
    fn extension_with_space_is_absorbed_into_stem() {
        // "t t" contains a space → not ASCII alphanumeric → absorbed
        assert_eq!(transform_filename("à faire.t t"), "a-faire-t-t");
    }

    #[test]
    fn extension_with_punctuation_is_absorbed_into_stem() {
        // "file.txt.bak" where "bak" is valid → "file-txt.bak"
        // But "file.txt?" where ext is "txt?" → "?" not alnum, absorbed
        // This would be: "notes.txt?" → ext "txt?" → absorbed → "notes-txt-"
        // Actually let's use a clearer case:
        // "data.2024!" → ext "2024!" → "!" not alnum → absorbed
        assert_eq!(transform_filename("data.2024!old"), "data-2024-old");
    }

    #[test]
    fn ascii_only_extension_under_ten_is_kept() {
        // Extensions that are purely ASCII alphanumeric and ≤ 10 stay
        assert_eq!(transform_filename("à faire .cuicui"), "a-faire.cuicui");
        assert_eq!(transform_filename("document.PDF"), "document.pdf");
        assert_eq!(transform_filename("image.JPEG"), "image.jpeg");
    }

    #[test]
    fn extension_exactly_ten_chars_is_kept() {
        assert_eq!(transform_filename("exact.abcdefghij"), "exact.abcdefghij");
    }

    #[test]
    fn extension_eleven_chars_is_absorbed() {
        assert_eq!(
            transform_filename("toolong.abcdefghijk"),
            "toolong-abcdefghijk"
        );
    }

    #[test]
    fn long_extension_with_mixed_stem_dots_is_absorbed() {
        // multi-dot file where the last segment exceeds 10 alnum chars
        assert_eq!(
            transform_filename("archive.backup.cuicuicuicui"),
            "archive-backup-cuicuicuicui"
        );
    }

    #[test]
    fn double_extensions_still_preserved() {
        // Double-ext check still happens before the 10-char rule
        assert_eq!(
            transform_filename("mon archive.TAR.GZ"),
            "mon-archive.tar.gz"
        );
        assert_eq!(
            transform_filename("mon  Archive.TaR.GZ"),
            "mon-archive.tar.gz"
        );
        assert_eq!(
            transform_filename("mon  Archive.TaR.Bz2"),
            "mon-archive.tar.bz2"
        );
        assert_eq!(
            transform_filename("mon  Archive.TaR .Bz2"),
            "mon-archive-tar.bz2"
        );
        assert_eq!(transform_filename("fichier.tar.bz2"), "fichier.tar.bz2");
    }

    #[test]
    fn compound_ext_with_long_last_segment_falls_through_to_single_ext_rule() {
        // "archive.tar.gzipconf" is NOT a known compound ext.
        // The last segment is "gzipconf" (8 alnum, ≤10) → valid single ext
        // file_stem = "archive.tar", ext = ".gzipconf"
        assert_eq!(
            transform_filename("archive.tar.gzipconf"),
            "archive-tar.gzipconf"
        );
    }

    // — Security / edge-case characters (audit additions) ────────────────────
    #[test]
    fn rtl_arabic_only_becomes_unnamed() {
        // Stem made entirely of non-Latin script collapses to empty → unnamed.
        assert_eq!(transform_filename("السلام"), "unnamed");
    }

    #[test]
    fn rtl_mixed_hebrew_and_latin_keeps_latin() {
        // Hebrew letters fall through to dashes; Latin survives.
        assert_eq!(transform_filename("hello שלום"), "hello");
    }

    #[test]
    fn zwj_emoji_family_collapses_around_latin() {
        // ZWJ (U+200D) is a Format char, not a combining mark — it must be
        // replaced by a dash, not silently absorbed, so adjacent emoji and
        // ZWJ collapse into a single dash and get trimmed.
        assert_eq!(
            transform_filename("family-\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}.txt"),
            "family.txt"
        );
    }

    #[test]
    fn variation_selector_is_treated_as_combining() {
        // U+FE0F (variation selector) has category Mn → dropped silently.
        // The remaining letters survive untouched.
        assert_eq!(transform_filename("file\u{FE0F}.txt"), "file.txt");
    }

    #[test]
    fn null_byte_in_stem_becomes_dash_no_panic() {
        // \0 is not alpha-num, not combining, no NFD decomposition → "-".
        // No panic is the primary assertion; output shape is incidental.
        assert_eq!(transform_filename("bad\0name.txt"), "bad-name.txt");
    }

    #[test]
    fn rtl_override_control_becomes_dash() {
        // U+202E (RIGHT-TO-LEFT OVERRIDE) is a Format char, not combining —
        // must NOT silently survive into the output filename.
        let out = transform_filename("file\u{202E}gnp.txt");
        assert!(
            !out.contains('\u{202E}'),
            "RTL override must not survive: got {out:?}"
        );
        assert_eq!(out, "file-gnp.txt");
    }

    #[test]
    fn path_traversal_segments_produce_no_slash_in_stem() {
        // Real filenames cannot contain '/' on Unix; this guards the pure
        // transformation against any accidental slash regression.
        let out = transform_stem("../../etc/passwd");
        assert!(
            !out.contains('/') && !out.contains('\\'),
            "transform_stem must never emit path separators: got {out:?}"
        );
        assert_eq!(out, "etc-passwd");
    }

    #[test]
    fn unnamed_collision_is_documented_and_deterministic() {
        // Two stems that reduce to "" share the same unnamed.<ext> destination.
        // This is the M1 finding from the audit: collision is detected later
        // by filter_conflicts in main.rs, but the transformation itself is
        // deterministic.
        assert_eq!(transform_filename("!!!.txt"), "unnamed.txt");
        assert_eq!(transform_filename("***.txt"), "unnamed.txt");
        assert_eq!(transform_filename("!!!.txt"), transform_filename("***.txt"));
    }

    #[test]
    fn nfd_long_mixed_matches_nfc() {
        use unicode_normalization::UnicodeNormalization;
        let nfc = "Café_Résumé_Élève";
        let nfd: String = nfc.nfd().collect();
        assert_ne!(nfc, nfd.as_str(), "test fixture must actually differ");
        assert_eq!(transform_stem(nfc), "cafe_resume_eleve");
        assert_eq!(transform_stem(&nfd), transform_stem(nfc));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// transform_dirname
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod transform_dirname_tests {
    use super::*;

    // Directories have no extension: a dot in the name is just a regular
    // separator and must be transliterated like any other character, not
    // preserved as a trailing ".ext".
    #[test]
    fn dot_in_dirname_is_not_treated_as_extension() {
        assert_eq!(transform_dirname("My Project.v2"), "my-project-v2");
        assert_eq!(
            transform_dirname("archive.2024.backup"),
            "archive-2024-backup"
        );
    }

    #[test]
    fn dirname_looking_like_a_file_keeps_no_extension() {
        // Would be "notes.txt" under transform_filename, but a directory
        // named "notes.txt" must become "notes-txt".
        assert_eq!(transform_dirname("notes.txt"), "notes-txt");
        assert_eq!(
            transform_dirname("Mon Dossier.tar.gz"),
            "mon-dossier-tar-gz"
        );
    }

    #[test]
    fn plain_dirname_is_slugified() {
        assert_eq!(transform_dirname("Mon Dossier"), "mon-dossier");
        assert_eq!(transform_dirname("Données 2024"), "donnees-2024");
        assert_eq!(transform_dirname("Réunion d'équipe"), "reunion-d-equipe");
    }

    #[test]
    fn hidden_dirname_is_not_modified() {
        assert_eq!(transform_dirname(".git"), ".git");
        assert_eq!(transform_dirname(".config"), ".config");
    }

    #[test]
    fn empty_after_transform_returns_unnamed() {
        assert_eq!(transform_dirname("!!!---!!!"), "unnamed");
        assert_eq!(transform_dirname("   "), "unnamed");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// fix_unicode (mojibake repair)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod fix_unicode_tests {
    use super::*;

    #[test]
    fn simple_latin1_mojibake_is_repaired() {
        assert_eq!(fix_unicode("CafÃ©"), "Café");
        assert_eq!(fix_unicode("RÃ©union d'Ã©quipe"), "Réunion d'équipe");
        assert_eq!(fix_unicode("DonnÃ©es Ã©tÃ©"), "Données été");
    }

    #[test]
    fn cp1252_mojibake_is_repaired() {
        // ’ (U+2019) encodes as E2 80 99 in UTF-8; misread as CP1252 → "â€™"
        assert_eq!(fix_unicode("Tomâ€™s"), "Tom’s");
        // œ (U+0153) encodes as C5 93; misread as CP1252 → "Å“"
        assert_eq!(fix_unicode("cÅ“ur"), "cœur");
    }

    #[test]
    fn double_mojibake_is_repaired() {
        // é → mojibake "Ã©" → mojibake again → "ÃƒÂ©"
        assert_eq!(fix_unicode("CafÃƒÂ©"), "Café");
    }

    #[test]
    fn correct_names_are_left_alone() {
        assert_eq!(fix_unicode("hello.txt"), "hello.txt");
        assert_eq!(fix_unicode("café"), "café");
        assert_eq!(fix_unicode("Réunion d'équipe"), "Réunion d'équipe");
        assert_eq!(fix_unicode("naïve straße"), "naïve straße");
    }

    #[test]
    fn names_with_chars_beyond_cp1252_are_left_alone() {
        assert_eq!(fix_unicode("photo 🦀"), "photo 🦀");
        assert_eq!(fix_unicode("中文"), "中文");
    }

    #[test]
    fn empty_string_is_left_alone() {
        assert_eq!(fix_unicode(""), "");
    }

    #[test]
    fn repair_is_idempotent() {
        let fixed = fix_unicode("CafÃ©");
        assert_eq!(fix_unicode(&fixed), fixed);
    }

    #[test]
    fn transform_filename_with_fix_unicode() {
        let opts = CleanupOptions {
            fix_unicode: true,
            ..CleanupOptions::default()
        };
        assert_eq!(
            transform_filename_with("CafÃ© MontrÃ©al.jpg", opts),
            "cafe-montreal.jpg"
        );
        // Without the fix, Ã and © go through the raw slug pipeline
        assert_eq!(
            transform_filename("CafÃ© MontrÃ©al.jpg"),
            "cafa-montra-al.jpg"
        );
    }

    #[test]
    fn transform_dirname_with_fix_unicode() {
        let opts = CleanupOptions {
            fix_unicode: true,
            ..CleanupOptions::default()
        };
        assert_eq!(
            transform_dirname_with("DonnÃ©es 2024", opts),
            "donnees-2024"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// fix_html (tag stripping + entity decoding)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod fix_html_tests {
    use super::*;

    #[test]
    fn inline_tags_are_stripped_without_a_gap() {
        assert_eq!(fix_html("<b>Hello</b> World"), "Hello World");
        assert_eq!(fix_html("client<b>s"), "clients");
    }

    #[test]
    fn block_tags_become_a_space() {
        assert_eq!(fix_html("<h1>Title</h1>"), " Title ");
        assert_eq!(fix_html("a<br/>b"), "a b");
        assert_eq!(fix_html("a<br>b"), "a b");
        assert_eq!(fix_html("un<p>deux"), "un deux");
        assert_eq!(fix_html("a<h3>b"), "a b");
        // The motivating example: block tag separates, inline tag welds.
        let opts = CleanupOptions {
            fix_html: true,
            ..CleanupOptions::default()
        };
        assert_eq!(
            transform_filename_with("données<br>client<b>s", opts),
            "donnees-clients"
        );
    }

    #[test]
    fn tags_with_attributes_are_stripped() {
        assert_eq!(fix_html("<a href=\"index.html\">Link</a>"), "Link");
        assert_eq!(fix_html("<img src='photo.jpg' alt='x'>legend"), "legend");
    }

    #[test]
    fn comments_and_doctype_are_stripped() {
        assert_eq!(fix_html("a<!-- note -->b"), "ab");
        assert_eq!(fix_html("<!DOCTYPE html>page"), "page");
    }

    #[test]
    fn lone_angle_brackets_are_kept() {
        // '<' not followed by a letter, '/' or '!' is not a tag
        assert_eq!(fix_html("a < b"), "a < b");
        assert_eq!(fix_html("1<2 et 3>2"), "1<2 et 3>2");
    }

    #[test]
    fn unclosed_tag_is_kept_verbatim() {
        assert_eq!(fix_html("foo<b.txt"), "foo<b.txt");
    }

    #[test]
    fn core_entities_are_decoded() {
        assert_eq!(fix_html("Tom &amp; Jerry"), "Tom & Jerry");
        assert_eq!(fix_html("1 &lt; 2 &gt; 0"), "1 < 2 > 0");
        assert_eq!(fix_html("l&apos;heure"), "l'heure");
        assert_eq!(fix_html("a&nbsp;b"), "a b");
    }

    #[test]
    fn accented_entities_are_decoded() {
        assert_eq!(fix_html("&eacute;t&eacute;"), "été");
        assert_eq!(fix_html("gar&ccedil;on"), "garçon");
        assert_eq!(fix_html("stra&szlig;e"), "straße");
        // Uppercase named entities fall back to the lowercase table
        assert_eq!(fix_html("&Eacute;t&eacute;"), "été");
    }

    #[test]
    fn numeric_entities_are_decoded() {
        assert_eq!(fix_html("&#233;t&#233;"), "été");
        assert_eq!(fix_html("&#xE9;t&#xe9;"), "été");
    }

    #[test]
    fn unknown_or_malformed_entities_are_kept() {
        assert_eq!(fix_html("&zzz;"), "&zzz;");
        assert_eq!(fix_html("Tom & Jerry"), "Tom & Jerry");
        assert_eq!(fix_html("100&"), "100&");
        // Numeric entity for a control character is refused
        assert_eq!(fix_html("&#0;"), "&#0;");
    }

    #[test]
    fn over_long_entity_body_is_kept_verbatim() {
        // The scanner only decodes an entity when the body (between `&` and `;`)
        // is at most MAX_ENTITY_LEN (12) bytes. A numeric entity padded with
        // leading zeros to exactly 12 bytes still decodes (0xE9 = 'é')...
        let at_cap = format!("&#x{:010X};", 0xE9_u32); // body "#x00000000E9" = 12 bytes
        assert_eq!(fix_html(&at_cap), "é");
        // ...but one byte over the cap is left untouched even though its value
        // would otherwise decode — the guard stops the search, not the value.
        let over_cap = format!("&#x{:011X};", 0xE9_u32); // 13-byte body
        assert_eq!(fix_html(&over_cap), over_cap);
    }

    #[test]
    fn out_of_range_numeric_entity_is_kept_verbatim() {
        // U+110000 is one past the highest Unicode scalar value; char::from_u32
        // rejects it, so the entity is kept verbatim instead of emitting garbage.
        assert_eq!(fix_html("&#x110000;"), "&#x110000;");
        // A decimal code point that overflows u32 is likewise rejected (the
        // radix parse fails before char conversion is even attempted).
        assert_eq!(fix_html("&#4294967296;"), "&#4294967296;");
    }

    #[test]
    fn decoded_lt_does_not_create_a_new_tag() {
        // Tags are stripped before entities are decoded
        assert_eq!(fix_html("&lt;b&gt;text&lt;/b&gt;"), "<b>text</b>");
    }

    #[test]
    fn transform_filename_with_fix_html() {
        let opts = CleanupOptions {
            fix_html: true,
            ..CleanupOptions::default()
        };
        assert_eq!(
            transform_filename_with("<h1>Mon &Eacute;t&eacute;</h1>.txt", opts),
            "mon-ete.txt"
        );
        assert_eq!(
            transform_filename_with("Tom &amp; Jerry.mp4", opts),
            "tom-jerry.mp4"
        );
    }

    #[test]
    fn transform_dirname_with_fix_html() {
        let opts = CleanupOptions {
            fix_html: true,
            ..CleanupOptions::default()
        };
        assert_eq!(
            transform_dirname_with("<b>Mes Documents</b>", opts),
            "mes-documents"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CleanupOptions combinations
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod cleanup_options_tests {
    use super::*;

    #[test]
    fn default_applies_nothing() {
        let name = "CafÃ© &amp; <i>The</i>.txt";
        assert_eq!(
            transform_filename_with(name, CleanupOptions::default()),
            transform_filename(name)
        );
    }

    #[test]
    fn all_enables_every_fix() {
        let opts = CleanupOptions::all();
        assert!(opts.fix_unicode);
        assert!(opts.fix_html);
    }

    #[test]
    fn unicode_then_html_are_applied_in_order() {
        // Mojibake is repaired first, then the HTML markup is removed
        assert_eq!(
            transform_filename_with("CafÃ© &amp; <i>The</i>.txt", CleanupOptions::all()),
            "cafe-the.txt"
        );
    }

    #[test]
    fn hidden_files_skip_cleanup_too() {
        assert_eq!(
            transform_filename_with(".hidden<b>.txt", CleanupOptions::all()),
            ".hidden<b>.txt"
        );
    }
}
