//! UbyPort‑compatible transliteration module
//!
//! Name fields use strict ASCII sanitization; address and A-record fields use the
//! broader transliteration + allowed-character rules.

use std::borrow::Cow;

use phf::phf_map;

/// Official UbyPort transliteration table.
pub static TRANSLIT: phf::Map<char, &'static str> = phf_map! {
    'À' => "A", 'à' => "a",
    'Ã' => "A", 'ã' => "a",
    'Å' => "A", 'å' => "a",
    'Ā' => "A", 'ā' => "a",
    'Æ' => "AE", 'æ' => "ae",

    'Ḃ' => "B", 'ḃ' => "b",

    'Ĉ' => "C", 'ĉ' => "c",
    'Ċ' => "C", 'ċ' => "c",

    'Ḋ' => "D", 'ḋ' => "d",
    'Ð' => "D", 'ð' => "d",

    'È' => "E", 'è' => "e",
    'Ê' => "E", 'ê' => "e",
    'Ē' => "E", 'ē' => "e",
    'Ė' => "E", 'ė' => "e",

    'Ḟ' => "F", 'ḟ' => "f",

    'Ĝ' => "G", 'ĝ' => "g",
    'Ğ' => "G", 'ğ' => "g",
    'Ġ' => "G", 'ġ' => "g",
    'Ģ' => "G", 'ģ' => "g",

    'Ĥ' => "H", 'ĥ' => "h",
    'Ħ' => "H", 'ħ' => "h",

    'Ï' => "I", 'ï' => "i",
    'Ì' => "I", 'ì' => "i",
    'Ĩ' => "I", 'ĩ' => "i",
    'Ī' => "I", 'ī' => "i",
    'Į' => "I", 'į' => "i",
    'İ' => "I", 'ı' => "i",
    'Ĳ' => "IJ", 'ĳ' => "ij",

    'Ĵ' => "J", 'ĵ' => "j",

    'Ķ' => "K", 'ķ' => "k",
    'ĸ' => "K",

    'Ļ' => "L", 'ļ' => "l",

    'Ṁ' => "M", 'ṁ' => "m",

    'Ñ' => "N", 'ñ' => "n",
    'Ņ' => "N", 'ņ' => "n",
    'Ŋ' => "N", 'ŋ' => "n",

    'Ò' => "O", 'ò' => "o",
    'Õ' => "O", 'õ' => "o",
    'Ø' => "O", 'ø' => "o",
    'Ō' => "O", 'ō' => "o",
    'Œ' => "OE", 'œ' => "oe",

    'Ṗ' => "P", 'ṗ' => "p",

    'Ŗ' => "R", 'ŗ' => "r",

    'Ŝ' => "S", 'ŝ' => "s",
    'Ṡ' => "S", 'ṡ' => "s",
    'Ş' => "S", 'ş' => "s",
    'Ș' => "S", 'ș' => "s",
    'ß' => "SS",
    'ẞ' => "SS",

    'Ṫ' => "T", 'ṫ' => "t",
    'Ţ' => "T", 'ţ' => "t",
    'Ț' => "T", 'ț' => "t",
    'Ŧ' => "T", 'ŧ' => "t",
    'Þ' => "T", 'þ' => "t",

    'Ù' => "U", 'ù' => "u",
    'Û' => "U", 'û' => "u",
    'Ũ' => "U", 'ũ' => "u",
    'Ū' => "U", 'ū' => "u",
    'Ŭ' => "U", 'ŭ' => "u",
    'Ų' => "U", 'ų' => "u",

    'Ŵ' => "W", 'ŵ' => "w",
    'Ẁ' => "W", 'ẁ' => "w",
    'Ẃ' => "W", 'ẃ' => "w",
    'Ẅ' => "W", 'ẅ' => "w",

    'Ŷ' => "Y", 'ŷ' => "y",
    'Ÿ' => "Y", 'ÿ' => "y",
    'Ỳ' => "Y", 'ỳ' => "y",

    'º' => "o", 'ª' => "a",
};

/// Folds UbyPort "allowed" diacritics to ASCII for person name fields.
static NAME_FOLD: phf::Map<char, &'static str> = phf_map! {
    'Á' => "A", 'á' => "a",
    'Ą' => "A", 'ą' => "a",
    'Ä' => "A", 'ä' => "a",
    'Â' => "A", 'â' => "a",
    'Ă' => "A", 'ă' => "a",
    'Č' => "C", 'č' => "c",
    'Ć' => "C", 'ć' => "c",
    'Ç' => "C", 'ç' => "c",
    'Ď' => "D", 'ď' => "d",
    'Đ' => "D", 'đ' => "d",
    'É' => "E", 'é' => "e",
    'Ę' => "E", 'ę' => "e",
    'Ë' => "E", 'ë' => "e",
    'Ě' => "E", 'ě' => "e",
    'Í' => "I", 'í' => "i",
    'Î' => "I", 'î' => "i",
    'Ĺ' => "L", 'ĺ' => "l",
    'Ł' => "L", 'ł' => "l",
    'Ľ' => "L", 'ľ' => "l",
    'Ń' => "N", 'ń' => "n",
    'Ň' => "N", 'ň' => "n",
    'Ó' => "O", 'ó' => "o",
    'Ô' => "O", 'ô' => "o",
    'Ö' => "O", 'ö' => "o",
    'Ő' => "O", 'ő' => "o",
    'Ŕ' => "R", 'ŕ' => "r",
    'Ř' => "R", 'ř' => "r",
    'Š' => "S", 'š' => "s",
    'Ś' => "S", 'ś' => "s",
    'Ť' => "T", 'ť' => "t",
    'Ú' => "U", 'ú' => "u",
    'Ű' => "U", 'ű' => "u",
    'Ü' => "U", 'ü' => "u",
    'Ů' => "U", 'ů' => "u",
    'Ý' => "Y", 'ý' => "y",
    'Ž' => "Z", 'ž' => "z",
    'Ź' => "Z", 'ź' => "z",
    'Ż' => "Z", 'ż' => "z",
};

/// Allowed characters for address and A-record fields.
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
        | '0'..='9'
        | ' ' | '\'' | '-' | '.' | ',' | '/'
    )
}

/// Decode numeric HTML entities (e.g. `&#287;` → `ğ`) found in spreadsheet data.
pub fn decode_html_entities(input: &str) -> Cow<'_, str> {
    if !input.contains('&') {
        return Cow::Borrowed(input);
    }

    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '&' && i + 2 < chars.len() && chars[i + 1] == '#' {
            let start = i + 2;
            let (radix, num_start) = if start < chars.len() && (chars[start] == 'x' || chars[start] == 'X') {
                (16, start + 1)
            } else {
                (10, start)
            };

            let mut j = num_start;
            while j < chars.len() && chars[j] != ';' {
                j += 1;
            }

            if j < chars.len() {
                let num_str: String = chars[num_start..j].iter().collect();
                if let Ok(code) = u32::from_str_radix(&num_str, radix) {
                    if let Some(decoded) = char::from_u32(code) {
                        out.push(decoded);
                        i = j + 1;
                        continue;
                    }
                }
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    Cow::Owned(out)
}

/// True when a string is valid for UbyPort surname/first name fields.
pub fn is_valid_unl_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '\'' || c == '-')
}

/// Sanitize a person name for UbyPort UNL export: ASCII letters, space, apostrophe, hyphen only.
pub fn sanitize_name(input: &str) -> String {
    let decoded = decode_html_entities(input);
    let trimmed = decoded.trim();
    let mut out = String::with_capacity(trimmed.len());

    for c in trimmed.chars() {
        if let Some(rep) = TRANSLIT.get(&c) {
            out.push_str(rep);
        } else if let Some(rep) = NAME_FOLD.get(&c) {
            out.push_str(rep);
        } else if c.is_ascii_alphabetic() || c == ' ' || c == '\'' || c == '-' {
            out.push(c);
        }
    }

    out
}

/// Sanitize an address for UNL export (broader character set than names).
pub fn sanitize_address(input: &str) -> String {
    let decoded = decode_html_entities(input);
    transliterate(decoded.trim())
}

/// Strip whitespace from travel document and visa numbers.
pub fn sanitize_doc_number(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Apply UbyPort transliteration rules for address/A-record fields.
pub fn transliterate(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for c in input.chars() {
        if let Some(rep) = TRANSLIT.get(&c) {
            out.push_str(rep);
        } else if is_allowed(c) {
            out.push(c);
        } else {
            out.push(c);
        }
    }

    out
}
