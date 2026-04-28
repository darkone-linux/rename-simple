use rename_files::{transform_filename, transform_stem, transliterate_char};

// ─────────────────────────────────────────────────────────────────────────────
// transliterate_char
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod transliterate_char_tests {
    use super::*;

    #[test]
    fn ascii_lowercase_letters_are_preserved() {
        for c in 'a'..='z' {
            assert_eq!(transliterate_char(c), c.to_string().as_str(),
                "expected '{c}' to stay '{c}'");
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
            assert_eq!(transliterate_char(c), c.to_string().as_str(),
                "expected digit '{c}' to stay '{c}'");
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

    // — Separators ────────────────────────────────────────────────────────────
    #[test]
    fn space_becomes_dash() {
        assert_eq!(transliterate_char(' '), "-");
    }

    #[test]
    fn punctuation_becomes_dash() {
        for c in ['.', ',', ';', '!', '?', ':', '(', ')', '[', ']', '\'', '"', '/', '\\'] {
            assert_eq!(transliterate_char(c), "-", "failed for '{c}'");
        }
    }

    #[test]
    fn symbols_become_dash() {
        for c in ['@', '#', '$', '%', '&', '*', '+', '=', '~', '^', '`'] {
            assert_eq!(transliterate_char(c), "-", "failed for '{c}'");
        }
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
        assert_eq!(transform_filename("document.PDF"),   "document.pdf");
        assert_eq!(transform_filename("image.JPEG"),     "image.jpeg");
    }

    #[test]
    fn double_extension_tar_gz_is_kept_together() {
        assert_eq!(transform_filename("archive.TAR.GZ"),     "archive.tar.gz"); // known -> double ext
        assert_eq!(transform_filename("archive.TOR.GZ"),     "archive-tor.gz"); // unknown -> simple ext
        assert_eq!(transform_filename("Mon Archive.tar.gz"), "mon-archive.tar.gz");
        assert_eq!(transform_filename("backup.TAR.BZ2"),     "backup.tar.bz2");
        assert_eq!(transform_filename("backup.TAR.XZ"),      "backup.tar.xz");
        assert_eq!(transform_filename("backup.TAR.ZST"),     "backup.tar.zst");
    }

    #[test]
    fn no_extension_files_are_handled() {
        assert_eq!(transform_filename("README"), "readme");
        assert_eq!(transform_filename("Mon Fichier"), "mon-fichier");
    }

    #[test]
    fn extension_preserved_on_clean_name() {
        assert_eq!(transform_filename("notes.txt"),     "notes.txt");
        assert_eq!(transform_filename("data.csv"),      "data.csv");
        assert_eq!(transform_filename("report.docx"),   "report.docx");
    }

    // — Accented filenames ────────────────────────────────────────────────────
    #[test]
    fn accented_filename_with_extension() {
        assert_eq!(transform_filename("Réunion d'équipe.docx"), "reunion-d-equipe.docx");
        assert_eq!(transform_filename("procès-verbal.pdf"),     "proces-verbal.pdf");
        assert_eq!(transform_filename("Données 2024.xlsx"),     "donnees-2024.xlsx");
    }

    // — Separators and underscores in filenames ───────────────────────────────
    #[test]
    fn numbered_prefix_preserved() {
        assert_eq!(transform_filename("01_introduction.md"),    "01_introduction.md");
        assert_eq!(transform_filename("02_  Résumé.md"),        "02_resume.md");
    }

    #[test]
    fn spaces_around_underscore_in_filename() {
        assert_eq!(transform_filename("01_ Titre du chapitre.txt"), "01_titre-du-chapitre.txt");
    }

    // — Hidden files ──────────────────────────────────────────────────────────
    #[test]
    fn hidden_files_are_not_modified() {
        assert_eq!(transform_filename(".gitignore"),     ".gitignore");
        assert_eq!(transform_filename(".hidden file"),   ".hidden file");
        assert_eq!(transform_filename(".DS_Store"),      ".DS_Store");
    }

    // — Edge cases ────────────────────────────────────────────────────────────
    #[test]
    fn empty_stem_after_transform_returns_unnamed() {
        assert_eq!(transform_filename("!!!(((---)))!!.txt"), "unnamed.txt");
        assert_eq!(transform_filename("???.pdf"),            "unnamed.pdf");
    }

    #[test]
    fn filename_with_only_extension_separators() {
        assert_eq!(transform_filename("   .txt"), "unnamed.txt");
    }

    #[test]
    fn filename_with_numbers_and_caps() {
        assert_eq!(transform_filename("IMG_2024_VACANCES ÉTÉ.jpg"), "img_2024_vacances-ete.jpg");
    }

    #[test]
    fn filename_no_dash_before_extension() {
        // trailing separators in stem must be stripped before adding extension
        assert_eq!(transform_filename("hello---.txt"), "hello.txt");
        assert_eq!(transform_filename("hello___.txt"), "hello.txt");
    }
}
