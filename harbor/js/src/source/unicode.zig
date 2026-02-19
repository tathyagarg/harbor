const std = @import("std");
const root = @import("../root.zig");

// NOTE: https://tc39.es/ecma262/#prod-WhiteSpace
pub const WHITESPACE_CHARS = [_]root.CodePoint{
    0x0009, // Tab
    0x000B, // Vertical Tab
    0x000C, // Form Feed
    0xFEFF, // Byte Order Mark
    0x0020, // Space
    0x00A0, // No-break space
};

// NOTE: https://tc39.es/ecma262/#prod-LineTerminator
pub const LINE_TERMINATOR_CHARS = [_]root.CodePoint{
    0x000A, // Line Feed
    0x000D, // Carriage Return
    0x2028, // Line Separator
    0x2029, // Paragraph Separator
};

pub const IDENTIFIER_START_CHARS = [_]root.CodePoint{
    0x0024, // Dollar Sign
    0x005F, // Low Line
};

pub const IDENTIFIER_PART_CHARS = [_]root.CodePoint{
    0x0024, // Dollar Sign
};

pub const HIGH_SURROGATE_START = 0xD800;
pub const HIGH_SURROGATE_END = 0xDBFF;

pub const LOW_SURROGATE_START = 0xDC00;
pub const LOW_SURROGATE_END = 0xDFFF;

pub fn is_whitespace(cp: root.CodePoint) bool {
    return std.mem.indexOf(
        root.CodePoint,
        &WHITESPACE_CHARS,
        &[_]root.CodePoint{cp},
    ) != null;
}

pub fn is_line_terminator(cp: root.CodePoint) bool {
    return std.mem.indexOf(
        root.CodePoint,
        &LINE_TERMINATOR_CHARS,
        &[_]root.CodePoint{cp},
    ) != null;
}

pub fn is_leading_surrogate(unit: u16) bool {
    return unit >= HIGH_SURROGATE_START and unit <= HIGH_SURROGATE_END;
}

pub fn is_trailing_surrogate(unit: u16) bool {
    return unit >= LOW_SURROGATE_START and unit <= LOW_SURROGATE_END;
}

// NOTE: This is an approximation of checking if a code point has property "ID_Start"
pub fn is_unicode_id_start(cp: root.CodePoint) bool {
    return (cp >= 0x41 and cp <= 0x5A) or (cp >= 0x61 and cp <= 0x7A);
}

pub fn is_unicode_id_continue(cp: root.CodePoint) bool {
    return (cp >= 0x41 and cp <= 0x5A) or (cp >= 0x61 and cp <= 0x7A) or
        (cp >= 0x30 and cp <= 0x39) or
        cp == 0x200C or // Zero Width Non-Joiner
        cp == 0x200D; // Zero Width Joiner
}

pub fn is_identifier_start(cp: root.CodePoint) bool {
    return std.mem.indexOf(
        root.CodePoint,
        &IDENTIFIER_START_CHARS,
        &[_]root.CodePoint{cp},
    ) != null or is_unicode_id_start(cp);
}

pub fn is_identifier_part(cp: root.CodePoint) bool {
    return std.mem.indexOf(
        root.CodePoint,
        &IDENTIFIER_PART_CHARS,
        &[_]root.CodePoint{cp},
    ) != null or is_unicode_id_continue(cp);
}

pub fn is_hex_digit(cp: root.CodePoint) bool {
    return (cp >= 0x30 and cp <= 0x39) or (cp >= 0x41 and cp <= 0x46) or
        (cp >= 0x61 and cp <= 0x66);
}

pub fn is_unicode_escape_sequence(cps: [*]const root.CodePoint, len: usize) ?usize {
    if (len < 5) return null;
    if (cps[0] != '\\') return null;
    if (cps[1] != 'u') return null;

    // u{codepoint}
    if (cps[2] == '{') {
        if (cps[4] != '}') return null;

        return 5;
    }

    // uXXXX
    for (cps[2..5]) |cp| {
        if (!is_hex_digit(cp)) return null;
    }

    return 6;
}

pub fn is_decimal_digit(cp: root.CodePoint) bool {
    return cp >= 0x30 and cp <= 0x39;
}

pub fn is_non_octal_digit(cp: root.CodePoint) bool {
    return cp >= 0x38 and cp <= 0x39;
}

pub fn is_octal_digit(cp: root.CodePoint) bool {
    return cp >= 0x30 and cp <= 0x37;
}

pub fn is_exponent_indicator(cp: root.CodePoint) bool {
    return cp == 0x45 or cp == 0x65;
}

pub fn is_single_escape_character(cp: root.CodePoint) bool {
    return cp == 0x22 or // Double Quote
        cp == 0x27 or // Single Quote
        cp == 0x5C or // Backslash
        cp == 0x62 or // b
        cp == 0x66 or // f
        cp == 0x6E or // n
        cp == 0x72 or // r
        cp == 0x74 or // t
        cp == 0x76; // v
}

pub fn is_non_escape_character(cp: root.CodePoint) bool {
    return !is_escape_character(cp) and !is_line_terminator(cp);
}

pub fn is_escape_character(cp: root.CodePoint) bool {
    return is_single_escape_character(cp) or is_decimal_digit(cp) or cp == 0x78 or cp == 0x75; // 'x' or 'u'
}

pub fn is_character_escape_sequence(cp: root.CodePoint) bool {
    return is_single_escape_character(cp) or is_non_escape_character(cp);
}

pub fn get_corresponding_character_escape(cp: root.CodePoint) ?root.CodePoint {
    switch (cp) {
        0x22 => return '"',
        0x27 => return '\'',
        0x5C => return '\\',
        0x62 => return '\u{0008}',
        0x66 => return '\u{000C}',
        0x6E => return '\n',
        0x72 => return '\r',
        0x74 => return '\t',
        0x76 => return '\u{000B}',
        else => return null,
    }
}

pub fn is_zero_to_three(cp: root.CodePoint) bool {
    return cp >= 0x30 and cp <= 0x33;
}

pub fn is_four_to_seven(cp: root.CodePoint) bool {
    return cp >= 0x34 and cp <= 0x37;
}
