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
