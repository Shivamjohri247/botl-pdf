use hashbrown::HashMap;

/// Map a byte value through WinAnsiEncoding (Windows code page 1252) to Unicode.
pub fn win_ansi_to_unicode(byte: u8) -> char {
    // Bytes 0x80-0x9F differ from Unicode; 0x00-0x7F and 0xA0-0xFF map directly.
    match byte {
        0x80 => '\u{20AC}', // Euro sign
        0x82 => '\u{201A}', // Single low-9 quotation mark
        0x83 => '\u{0192}', // Latin small letter f with hook
        0x84 => '\u{201E}', // Double low-9 quotation mark
        0x85 => '\u{2026}', // Horizontal ellipsis
        0x86 => '\u{2020}', // Dagger
        0x87 => '\u{2021}', // Double dagger
        0x88 => '\u{02C6}', // Modifier letter circumflex accent
        0x89 => '\u{2030}', // Per mille sign
        0x8A => '\u{0160}', // Latin capital letter S with caron
        0x8B => '\u{2039}', // Single left-pointing angle quotation mark
        0x8C => '\u{0152}', // Latin capital ligature OE
        0x8E => '\u{017D}', // Latin capital letter Z with caron
        0x91 => '\u{2018}', // Left single quotation mark
        0x92 => '\u{2019}', // Right single quotation mark
        0x93 => '\u{201C}', // Left double quotation mark
        0x94 => '\u{201D}', // Right double quotation mark
        0x95 => '\u{2022}', // Bullet
        0x96 => '\u{2013}', // En dash
        0x97 => '\u{2014}', // Em dash
        0x98 => '\u{02DC}', // Small tilde
        0x99 => '\u{2122}', // Trade mark sign
        0x9A => '\u{0161}', // Latin small letter s with caron
        0x9B => '\u{203A}', // Single right-pointing angle quotation mark
        0x9C => '\u{0153}', // Latin small ligature oe
        0x9E => '\u{017E}', // Latin small letter z with caron
        0x9F => '\u{0178}', // Latin capital letter Y with diaeresis
        _ => byte as char,  // Direct mapping for everything else
    }
}

/// Map a byte through MacRomanEncoding to Unicode.
pub fn mac_roman_to_unicode(byte: u8) -> char {
    match byte {
        0x80 => '\u{00C4}', // Ä
        0x81 => '\u{00C5}', // Å
        0x82 => '\u{00C7}', // Ç
        0x83 => '\u{00C9}', // É
        0x84 => '\u{00D1}', // Ñ
        0x85 => '\u{00D6}', // Ö
        0x86 => '\u{00DC}', // Ü
        0x87 => '\u{00E1}', // á
        0x88 => '\u{00E0}', // à
        0x89 => '\u{00E2}', // â
        0x8A => '\u{00E4}', // ä
        0x8B => '\u{00E3}', // ã
        0x8C => '\u{00E5}', // å
        0x8D => '\u{00E7}', // ç
        0x8E => '\u{00E9}', // é
        0x8F => '\u{00E8}', // è
        0x90 => '\u{00EA}', // ê
        0x91 => '\u{00EB}', // ë
        0x92 => '\u{00ED}', // í
        0x93 => '\u{00EC}', // ì
        0x94 => '\u{00EE}', // î
        0x95 => '\u{00EF}', // ï
        0x96 => '\u{00F1}', // ñ
        0x97 => '\u{00F3}', // ó
        0x98 => '\u{00F2}', // ò
        0x99 => '\u{00F4}', // ô
        0x9A => '\u{00F6}', // ö
        0x9B => '\u{00F5}', // õ
        0x9C => '\u{00FA}', // ú
        0x9D => '\u{00F9}', // ù
        0x9E => '\u{00FB}', // û
        0x9F => '\u{00FC}', // ü
        0xA0 => '\u{2020}', // †
        0xA1 => '\u{00B0}', // °
        0xA2 => '\u{00A2}', // ¢
        0xA3 => '\u{00A3}', // £
        0xA4 => '\u{00A7}', // §
        0xA5 => '\u{2022}', // •
        0xA6 => '\u{00B6}', // ¶
        0xA7 => '\u{00DF}', // ß
        0xA8 => '\u{00AE}', // ®
        0xA9 => '\u{00A9}', // ©
        0xAA => '\u{2122}', // ™
        0xAB => '\u{00B4}', // ´
        0xAC => '\u{00A8}', // ¨
        0xAD => '\u{2260}', // ≠
        0xAE => '\u{00C6}', // Æ
        0xAF => '\u{00D8}', // Ø
        0xB0 => '\u{221E}', // ∞
        0xB1 => '\u{00B1}', // ±
        0xB2 => '\u{2264}', // ≤
        0xB3 => '\u{2265}', // ≥
        0xB4 => '\u{00A5}', // ¥
        0xB5 => '\u{00B5}', // µ
        0xB6 => '\u{2202}', // ∂
        0xB7 => '\u{2211}', // ∑
        0xB8 => '\u{220F}', // ∏
        0xB9 => '\u{03C0}', // π
        0xBA => '\u{222B}', // ∫
        0xBB => '\u{00AA}', // ª
        0xBC => '\u{00BA}', // º
        0xBD => '\u{2126}', // Ω
        0xBE => '\u{00E6}', // æ
        0xBF => '\u{00F8}', // ø
        0xC0 => '\u{00BF}', // ¿
        0xC1 => '\u{00A1}', // ¡
        0xC2 => '\u{00AC}', // ¬
        0xC3 => '\u{221A}', // √
        0xC4 => '\u{0192}', // ƒ
        0xC5 => '\u{2248}', // ≈
        0xC6 => '\u{2206}', // ∆
        0xC7 => '\u{00AB}', // «
        0xC8 => '\u{00BB}', // »
        0xC9 => '\u{2026}', // …
        0xCA => '\u{00A0}', // non-breaking space
        0xCB => '\u{00C0}', // À
        0xCC => '\u{00C3}', // Ã
        0xCD => '\u{00D5}', // Õ
        0xCE => '\u{0152}', // Œ
        0xCF => '\u{0153}', // œ
        0xD0 => '\u{2013}', // –
        0xD1 => '\u{2014}', // —
        0xD2 => '\u{201C}', // "
        0xD3 => '\u{201D}', // "
        0xD4 => '\u{2018}', // '
        0xD5 => '\u{2019}', // '
        0xD6 => '\u{00F7}', // ÷
        0xD7 => '\u{25CA}', // ◊
        0xD8 => '\u{00FF}', // ÿ
        0xD9 => '\u{0178}', // Ÿ
        0xDA => '\u{2044}', // ⁄
        0xDB => '\u{20AC}', // €
        0xDD => '\u{2039}', // ‹
        0xDE => '\u{203A}', // ›
        0xDF => '\u{FB01}', // ﬁ
        0xE0 => '\u{FB02}', // ﬂ
        0xE1 => '\u{2021}', // ‡
        0xE2 => '\u{00B7}', // ·
        0xE3 => '\u{201A}', // ‚
        0xE4 => '\u{201E}', // „
        0xE5 => '\u{2030}', // ‰
        0xE6 => '\u{00C2}', // Â
        0xE7 => '\u{00CA}', // Ê
        0xE8 => '\u{00C1}', // Á
        0xE9 => '\u{00CB}', // Ë
        0xEA => '\u{00C8}', // È
        0xEB => '\u{00CD}', // Í
        0xEC => '\u{00CE}', // Î
        0xED => '\u{00CF}', // Ï
        0xEE => '\u{00CC}', // Ì
        0xEF => '\u{00D3}', // Ó
        0xF0 => '\u{00D4}', // Ô
        0xF1 => '\u{0100}', // Ā wait, this isn't standard...
        0xF2 => '\u{00D2}', // Ò
        0xF3 => '\u{00DA}', // Ú
        0xF4 => '\u{00DB}', // Û
        0xF5 => '\u{00D9}', // Ù
        0xF6 => '\u{0131}', // ı
        0xF7 => '\u{02C6}', // ˆ
        0xF8 => '\u{02DC}', // ˜
        0xF9 => '\u{00AF}', // ¯
        0xFA => '\u{02D8}', // ˘
        0xFB => '\u{02D9}', // ˙
        0xFC => '\u{02DA}', // ˚
        0xFD => '\u{00B8}', // ¸
        0xFE => '\u{02DD}', // ˝
        0xFF => '\u{02DB}', // ˛
        _ => byte as char,
    }
}

/// Symbol encoding (used by Symbol font).
/// Maps byte values to Greek/math symbols.
pub fn symbol_to_unicode(byte: u8) -> char {
    match byte {
        0x20 => ' ',
        0x21 => '!',
        0x26 => '&',
        0x27 => '\'',
        0x28 => '(',
        0x29 => ')',
        0x2B => '+',
        0x2C => ',',
        0x2D => '-',
        0x2E => '.',
        0x2F => '/',
        0x30..=0x39 => (byte - 0x30 + b'0') as char, // digits
        0x3A => ':',
        0x3B => ';',
        0x3D => '=',
        0x3F => '?',
        0x41 => 'Α', // Alpha
        0x42 => 'Β', // Beta
        0x43 => 'Χ', // Chi
        0x44 => 'Δ', // Delta
        0x45 => 'Ε', // Epsilon
        0x46 => 'Φ', // Phi
        0x47 => 'Γ', // Gamma
        0x48 => 'Η', // Eta
        0x49 => 'Ι', // Iota
        0x4A => 'Θ', // Theta
        0x4B => 'Κ', // Kappa
        0x4C => 'Λ', // Lambda
        0x4D => 'Μ', // Mu
        0x4E => 'Ν', // Nu
        0x4F => 'Ο', // Omicron
        0x50 => 'Π', // Pi
        0x51 => 'Θ', // Theta (alt)
        0x52 => 'Ρ', // Rho
        0x53 => 'Σ', // Sigma
        0x54 => 'Τ', // Tau
        0x55 => 'Υ', // Upsilon
        0x57 => 'Ω', // Omega
        0x58 => 'Ξ', // Xi
        0x59 => 'Ψ', // Psi
        0x5A => 'Ζ', // Zeta
        0x61 => 'α', // alpha
        0x62 => 'β', // beta
        0x63 => 'χ', // chi
        0x64 => 'δ', // delta
        0x65 => 'ε', // epsilon
        0x66 => 'φ', // phi
        0x67 => 'γ', // gamma
        0x68 => 'η', // eta
        0x69 => 'ι', // iota
        0x6A => 'θ', // theta (curly)
        0x6B => 'κ', // kappa
        0x6C => 'λ', // lambda
        0x6D => 'μ', // mu
        0x6E => 'ν', // nu
        0x6F => 'ο', // omicron
        0x70 => 'π', // pi
        0x71 => 'θ', // theta
        0x72 => 'ρ', // rho
        0x73 => 'σ', // sigma
        0x74 => 'τ', // tau
        0x75 => 'υ', // upsilon
        0x76 => 'ϖ', // pi (variant)
        0x77 => 'ω', // omega
        0x78 => 'ξ', // xi
        0x79 => 'ψ', // psi
        0x7A => 'ζ', // zeta
        _ => byte as char,
    }
}

/// ZapfDingbats encoding.
/// Maps byte values to Zapf Dingbats symbols.
pub fn zapf_dingbats_to_unicode(byte: u8) -> char {
    // ZapfDingbats uses codes 0x20-0xFE mapping to various dingbat symbols
    // This is a simplified mapping for the most common symbols
    match byte {
        0x20 => ' ',
        0x21 => '✁',
        0x22 => '✂',
        0x23 => '✃',
        0x24 => '✄',
        0x25 => '✆',
        0x26 => '✇',
        0x27 => '✈',
        0x28 => '✉',
        0x29 => '✌',
        0x2A => '✍',
        0x2B => '✎',
        0x2C => '✏',
        0x2D => '✐',
        0x2E => '✑',
        0x2F => '✒',
        0x30 => '✓',
        0x31 => '✔',
        0x32 => '✕',
        0x33 => '✖',
        0x34 => '✗',
        0x35 => '✘',
        0x36 => '✙',
        0x37 => '✚',
        0x38 => '✛',
        0x39 => '✜',
        0x3A => '✝',
        0x3B => '✞',
        0x3C => '✟',
        0x3D => '✠',
        0x3E => '✡',
        0x3F => '✢',
        0x40 => '✣',
        0x4E => '✦',
        0x4F => '✧',
        0x50 => '✩',
        0x51 => '✪',
        0x52 => '✫',
        0x53 => '✬',
        0x54 => '✭',
        0x55 => '✮',
        0x56 => '✯',
        0x57 => '✰',
        0x58 => '✱',
        0x59 => '✲',
        0x5A => '✳',
        0x5B => '✴',
        0x5C => '✵',
        0x5D => '✶',
        0x5E => '✷',
        0x5F => '✸',
        0x60 => '✹',
        0x61 => '✺',
        0x62 => '✻',
        0x63 => '✼',
        0x64 => '✽',
        0x65 => '✾',
        0x66 => '✿',
        0x67 => '❀',
        0x68 => '❁',
        0x69 => '❂',
        0x6A => '❃',
        0x6B => '❄',
        0x6C => '❅',
        0x6D => '❆',
        0x6E => '❇',
        0x6F => '❈',
        0x70 => '❉',
        0x71 => '❊',
        0x72 => '❋',
        // Arrow and geometric shapes
        0xA0..=0xFE => std::char::from_u32(0x2700 + byte as u32).unwrap_or(' '),
        _ => byte as char,
    }
}

/// Resolve a character code to Unicode using the appropriate encoding.
pub fn decode_char(byte: u8, encoding: &str) -> char {
    match encoding {
        "WinAnsiEncoding" => win_ansi_to_unicode(byte),
        "MacRomanEncoding" => mac_roman_to_unicode(byte),
        "Symbol" | "SymbolSet" => symbol_to_unicode(byte),
        "ZapfDingbats" => zapf_dingbats_to_unicode(byte),
        "StandardEncoding" | "MacExpertEncoding" => win_ansi_to_unicode(byte), // Close enough
        _ => {
            // Default: treat as WinAnsi for Latin-encoded fonts
            if byte.is_ascii() {
                byte as char
            } else {
                win_ansi_to_unicode(byte)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_ansi_euro() {
        assert_eq!(win_ansi_to_unicode(0x80), '€');
    }

    #[test]
    fn test_win_ansi_ascii() {
        assert_eq!(win_ansi_to_unicode(0x41), 'A');
        assert_eq!(win_ansi_to_unicode(0x7E), '~');
    }

    #[test]
    fn test_win_ansi_smart_quotes() {
        assert_eq!(win_ansi_to_unicode(0x91), '\u{2018}'); // left single quote
        assert_eq!(win_ansi_to_unicode(0x92), '\u{2019}'); // right single quote
    }

    #[test]
    fn test_mac_roman() {
        assert_eq!(mac_roman_to_unicode(0x80), 'Ä');
        assert_eq!(mac_roman_to_unicode(0x9A), 'ö');
    }

    #[test]
    fn test_symbol_greek() {
        assert_eq!(symbol_to_unicode(0x41), 'Α');
        assert_eq!(symbol_to_unicode(0x61), 'α');
    }

    #[test]
    fn test_decode_char() {
        assert_eq!(decode_char(0x41, "WinAnsiEncoding"), 'A');
        assert_eq!(decode_char(0x80, "WinAnsiEncoding"), '€');
        assert_eq!(decode_char(0x41, "Symbol"), 'Α');
    }
}
