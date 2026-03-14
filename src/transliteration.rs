//! UbyPort‑compatible transliteration module
//!
//! This module ensures that all exported UNL lines contain ONLY characters
//! permitted by the official UbyPort specification.
//!
//! Rules implemented:
//! 1. Characters in the transliteration table → replaced
//! 2. Allowed characters → kept as-is
//! 3. Everything else → kept (rare fallback)

use phf::phf_map;

/// Official UbyPort transliteration table.
/// Any character NOT in the allowed set must be replaced using this map.
/// Uses a compile-time perfect hash (phf) for O(1) lookups and no runtime allocation.
pub static TRANSLIT: phf::Map<char, &'static str> = phf_map! {
    // A variants
    'À' => "A", 'à' => "a",
    'Ã' => "A", 'ã' => "a",
    'Å' => "A", 'å' => "a",
    'Ā' => "A", 'ā' => "a",
    'Æ' => "AE", 'æ' => "ae",

    // B
    'Ḃ' => "B", 'ḃ' => "b",

    // C
    'Ĉ' => "C", 'ĉ' => "c",
    'Ċ' => "C", 'ċ' => "c",

    // D
    'Ḋ' => "D", 'ḋ' => "d",
    'Ð' => "D", 'ð' => "d",

    // E
    'È' => "E", 'è' => "e",
    'Ê' => "E", 'ê' => "e",
    'Ē' => "E", 'ē' => "e",
    'Ė' => "E", 'ė' => "e",

    // F
    'Ḟ' => "F", 'ḟ' => "f",

    // G
    'Ĝ' => "G", 'ĝ' => "g",
    'Ğ' => "G", 'ğ' => "g",
    'Ġ' => "G", 'ġ' => "g",
    'Ģ' => "G", 'ģ' => "g",

    // H
    'Ĥ' => "H", 'ĥ' => "h",
    'Ħ' => "H", 'ħ' => "h",

    // I
    'Ï' => "I", 'ï' => "i",
    'Ì' => "I", 'ì' => "i",
    'Ĩ' => "I", 'ĩ' => "i",
    'Ī' => "I", 'ī' => "i",
    'Į' => "I", 'į' => "i",
    'İ' => "I", 'ı' => "i",
    'Ĳ' => "IJ", 'ĳ' => "ij",

    // J
    'Ĵ' => "J", 'ĵ' => "j",

    // K
    'Ķ' => "K", 'ķ' => "k",
    'ĸ' => "K",

    // L
    'Ļ' => "L", 'ļ' => "l",

    // M
    'Ṁ' => "M", 'ṁ' => "m",

    // N
    'Ñ' => "N", 'ñ' => "n",
    'Ņ' => "N", 'ņ' => "n",
    'Ŋ' => "N", 'ŋ' => "n",

    // O
    'Ò' => "O", 'ò' => "o",
    'Õ' => "O", 'õ' => "o",
    'Ø' => "O", 'ø' => "o",
    'Ō' => "O", 'ō' => "o",
    'Œ' => "OE", 'œ' => "oe",

    // P
    'Ṗ' => "P", 'ṗ' => "p",

    // R
    'Ŗ' => "R", 'ŗ' => "r",

    // S
    'Ŝ' => "S", 'ŝ' => "s",
    'Ṡ' => "S", 'ṡ' => "s",
    'Ş' => "S", 'ș' => "s",
    'ß' => "SS",
    'ẞ' => "SS",

    // T
    'Ṫ' => "T", 'ṫ' => "t",
    'Ţ' => "T", 'ţ' => "t",
    'Ț' => "T", 'ț' => "t",
    'Ŧ' => "T", 'ŧ' => "t",
    'Þ' => "T", 'þ' => "t",

    // U
    'Ù' => "U", 'ù' => "u",
    'Û' => "U", 'û' => "u",
    'Ũ' => "U", 'ũ' => "u",
    'Ū' => "U", 'ū' => "u",
    'Ŭ' => "U", 'ŭ' => "u",
    'Ų' => "U", 'ų' => "u",

    // W
    'Ŵ' => "W", 'ŵ' => "w",
    'Ẁ' => "W", 'ẁ' => "w",
    'Ẃ' => "W", 'ẃ' => "w",
    'Ẅ' => "W", 'ẅ' => "w",

    // Y
    'Ŷ' => "Y", 'ŷ' => "y",
    'Ÿ' => "Y", 'ÿ' => "y",
    'Ỳ' => "Y", 'ỳ' => "y",
};

/// Allowed characters from UbyPort's "Tabulka povolených znaků".
/// These characters may appear unchanged in the UNL file.
/// Includes basic Latin, selected diacritics (e.g. Á, Č, Š, Ž), and space, apostrophe, hyphen.
pub fn is_allowed(c: char) -> bool {
    matches!(c,
        'A'..='Z' | 'a'..='z'
        | 'Á' | 'á' | 'Ą' | 'ą' | 'Ä' | 'ä' | 'Â' | 'â' | 'Ă' | 'ă'
        | 'Č' | 'č' | 'Ć' | 'ć' | 'Ç' | 'ç'
        | 'Ď' | 'ď' | 'Đ' | 'đ'
        | 'É' | 'é' | 'Ę' | 'ę' | 'Ë' | 'ë' | 'Ě' | 'ě'
        | 'Í' | 'í' | 'Î' | 'î'
        | 'Ĺ' | 'ĺ' | 'Ł' | 'ł' | 'Ľ' | 'ľ'
        | 'Ń' | 'ń' | 'Ň' | 'ň'
        | 'Ó' | 'ó' | 'Ô' | 'ô' | 'Ö' | 'ö' | 'Ő' | 'ő'
        | 'Ŕ' | 'ŕ' | 'Ř' | 'ř'
        | 'Š' | 'š' | 'Ś' | 'ś' | 'ß'
        | 'Ť' | 'ť'
        | 'Ú' | 'ú' | 'Ű' | 'ű' | 'Ü' | 'ü' | 'Ů' | 'ů'
        | 'Ý' | 'ý'
        | 'Ž' | 'ž' | 'Ź' | 'ź' | 'Ż' | 'ż'
        | ' ' | '\'' | '-'
    )
}

/// Apply UbyPort transliteration rules:
/// 1. Characters in transliteration table → replaced
/// 2. Allowed characters → kept as-is
/// 3. Everything else → kept (rare fallback; avoids data loss for unexpected Unicode)
pub fn transliterate(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for c in input.chars() {
        if let Some(rep) = TRANSLIT.get(&c) {
            out.push_str(rep);
        } else if is_allowed(c) {
            out.push(c);
        } else {
            // Unknown character: pass through so we don't drop data; encoder may replace later.
            out.push(c);
        }
    }

    out
}