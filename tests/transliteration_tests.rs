//! Integration tests for UbyPort transliteration and name sanitization.

use guest_checkin::transliteration::{
    decode_html_entities, is_allowed, is_valid_unl_name, sanitize_address, sanitize_doc_number,
    sanitize_name, transliterate, TRANSLIT,
};

#[test]
fn test_allowed_characters_pass_through() {
    let input = "ÁáČčĎďÉéÍíÓóÚúÝýŽž";
    let output = transliterate(input);
    assert_eq!(output, input);
}

#[test]
fn test_basic_transliterations() {
    assert_eq!(transliterate("ñ"), "n");
    assert_eq!(transliterate("Ñ"), "N");
    assert_eq!(transliterate("ß"), "SS");
    assert_eq!(transliterate("Æ"), "AE");
    assert_eq!(transliterate("æ"), "ae");
    assert_eq!(transliterate("Ø"), "O");
    assert_eq!(transliterate("ø"), "o");
    assert_eq!(transliterate("Þ"), "T");
    assert_eq!(transliterate("þ"), "t");
}

#[test]
fn test_multi_character_transliterations() {
    assert_eq!(transliterate("Œ"), "OE");
    assert_eq!(transliterate("œ"), "oe");
    assert_eq!(transliterate("Ĳ"), "IJ");
    assert_eq!(transliterate("ĳ"), "ij");
}

#[test]
fn test_full_name_transliteration() {
    let input = "Castañeda Solórzano";
    let expected = "Castaneda Solórzano";
    assert_eq!(transliterate(input), expected);
}

#[test]
fn test_mixed_string() {
    let input = "Ægir Þór Ñandú";
    let expected = "AEgir Tór Nandú";
    assert_eq!(transliterate(input), expected);
}

#[test]
fn test_is_allowed_true_cases() {
    for c in ['A', 'z', 'Á', 'á', 'Č', 'č', 'Ó', 'ó', 'ß', ' '] {
        assert!(is_allowed(c), "Character {} should be allowed", c);
    }
}

#[test]
fn test_is_allowed_false_cases() {
    for c in ['ñ', 'Ñ', 'Æ', 'ø', 'Þ'] {
        assert!(!is_allowed(c), "Character {} should NOT be allowed", c);
    }
}

#[test]
fn test_transliteration_table_completeness() {
    for (key, val) in TRANSLIT.entries() {
        let input = key.to_string();
        let output = transliterate(&input);
        assert_eq!(output, *val, "Transliteration mismatch for {}", key);
    }
}

#[test]
fn test_no_panic_on_unknown_unicode() {
    let input = "𐍈𐍈𐍈";
    let output = transliterate(input);
    assert_eq!(output, input);
}

// --- sanitize_name regression tests (issue #36 examples) ---

#[test]
fn test_sanitize_name_swedish() {
    assert_eq!(sanitize_name("Franzén"), "Franzen");
    assert_eq!(sanitize_name("Esbjörn"), "Esbjorn");
    assert_eq!(sanitize_name("Höjer"), "Hojer");
    assert_eq!(sanitize_name("Arvid"), "Arvid");
}

#[test]
fn test_sanitize_name_turkish() {
    assert_eq!(sanitize_name("Kesecioğlu Güvenç"), "Kesecioglu Guvenc");
    assert_eq!(sanitize_name("Kesecio&#287;lu G&#252;ven&#231;"), "Kesecioglu Guvenc");
    assert_eq!(sanitize_name("Öznur"), "Oznur");
}

#[test]
fn test_sanitize_name_portuguese_brazilian() {
    assert_eq!(sanitize_name("Gonçalves"), "Goncalves");
    assert_eq!(sanitize_name("André"), "Andre");
    assert_eq!(sanitize_name("Vitória"), "Vitoria");
}

#[test]
fn test_sanitize_name_spanish() {
    assert_eq!(sanitize_name("Ortegón"), "Ortegon");
    assert_eq!(sanitize_name("Tesías"), "Tesias");
    assert_eq!(sanitize_name("César"), "Cesar");
    assert_eq!(sanitize_name("Gutiérrez Flores"), "Gutierrez Flores");
    assert_eq!(sanitize_name("José Enrique"), "Jose Enrique");
    assert_eq!(sanitize_name("González Martín"), "Gonzalez Martin");
    assert_eq!(sanitize_name("Iván"), "Ivan");
    assert_eq!(sanitize_name("Diez López - Linares"), "Diez Lopez - Linares");
}

#[test]
fn test_sanitize_name_argentine() {
    assert_eq!(sanitize_name("Tomás"), "Tomas");
}

#[test]
fn test_sanitize_name_polish() {
    assert_eq!(sanitize_name("Szczepański"), "Szczepanski");
    assert_eq!(sanitize_name("Dąbrowski"), "Dabrowski");
}

#[test]
fn test_sanitize_name_french() {
    assert_eq!(sanitize_name("Zénobe"), "Zenobe");
}

#[test]
fn test_sanitize_name_trims_whitespace() {
    assert_eq!(sanitize_name("  Cervantes "), "Cervantes");
    assert_eq!(sanitize_name("Jimena "), "Jimena");
    assert_eq!(sanitize_name("Diez López - Linares "), "Diez Lopez - Linares");
}

#[test]
fn test_sanitize_name_turkish_s_cedilla() {
    assert_eq!(sanitize_name("Şahin"), "Sahin");
    assert_eq!(sanitize_name("şule"), "sule");
}

#[test]
fn test_is_valid_unl_name() {
    assert!(is_valid_unl_name("Goncalves"));
    assert!(is_valid_unl_name("O'Brien"));
    assert!(is_valid_unl_name("da Silva-Goncalves"));
    assert!(!is_valid_unl_name(""));
    assert!(!is_valid_unl_name("Gonçalves"));
}

#[test]
fn test_decode_html_entities() {
    assert_eq!(decode_html_entities("&#287;"), "ğ");
    assert_eq!(decode_html_entities("Kesecio&#287;lu"), "Kesecioğlu");
    assert_eq!(decode_html_entities("plain"), "plain");
}

#[test]
fn test_sanitize_doc_number_strips_spaces() {
    assert_eq!(sanitize_doc_number("33119559 3ZZ2"), "331195593ZZ2");
    assert_eq!(sanitize_doc_number("S25094413"), "S25094413");
}

#[test]
fn test_sanitize_address_ordinal() {
    assert_eq!(sanitize_address("2º dto"), "2o dto");
}
