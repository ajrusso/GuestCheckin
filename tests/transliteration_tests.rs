//! Integration tests for UbyPort transliteration.
//! Covers allowed-character pass-through, table replacements, multi-char expansions,
//! and edge cases (unknown Unicode, table consistency).

use guest_checkin::transliteration::transliterate;
use guest_checkin::transliteration::is_allowed;
use guest_checkin::transliteration::TRANSLIT;

/// Characters in the allowed set must be returned unchanged.
#[test]
fn test_allowed_characters_pass_through() {
    let input = "ÁáČčĎďÉéÍíÓóÚúÝýŽž";
    let output = transliterate(input);
    assert_eq!(output, input);
}

/// Single-character replacements from the transliteration table (e.g. ñ→n, ß→SS).
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

/// Characters that expand to two letters (e.g. Œ→OE, Ĳ→IJ).
#[test]
fn test_multi_character_transliterations() {
    assert_eq!(transliterate("Œ"), "OE");
    assert_eq!(transliterate("œ"), "oe");
    assert_eq!(transliterate("Ĳ"), "IJ");
    assert_eq!(transliterate("ĳ"), "ij");
}

/// Real-name example: mix of transliterated (ñ) and allowed (ó) characters.
#[test]
fn test_full_name_transliteration() {
    let input = "Castañeda Solórzano";
    let expected = "Castaneda Solórzano"; // ñ → n, ó stays ó
    assert_eq!(transliterate(input), expected);
}

/// Multiple replacements in one string (Æ, Þ, Ñ) with allowed chars (ó, ú) preserved.
#[test]
fn test_mixed_string() {
    let input = "Ægir Þór Ñandú";
    let expected = "AEgir Tór Nandú";
    assert_eq!(transliterate(input), expected);
}

/// is_allowed returns true for basic Latin, diacritics in the allowed set, and space/apostrophe/hyphen.
#[test]
fn test_is_allowed_true_cases() {
    for c in ['A', 'z', 'Á', 'á', 'Č', 'č', 'Ó', 'ó', 'ß', ' '] {
        assert!(is_allowed(c), "Character {} should be allowed", c);
    }
}

/// is_allowed returns false for characters that must be transliterated (e.g. ñ, Æ).
#[test]
fn test_is_allowed_false_cases() {
    for c in ['ñ', 'Ñ', 'Æ', 'ø', 'Þ'] {
        assert!(!is_allowed(c), "Character {} should NOT be allowed", c);
    }
}

/// Every entry in TRANSLIT must map to its replacement when run through transliterate().
#[test]
fn test_transliteration_table_completeness() {
    for (key, val) in TRANSLIT.entries() {
        let input = key.to_string();
        let output = transliterate(&input);
        assert_eq!(output, *val, "Transliteration mismatch for {}", key);
    }
}

/// Characters not in TRANSLIT and not in the allowed set pass through unchanged (no panic, no drop).
#[test]
fn test_no_panic_on_unknown_unicode() {
    let input = "𐍈𐍈𐍈"; // Gothic letter faihu — not in table or allowed set
    let output = transliterate(input);
    assert_eq!(output, input);
}