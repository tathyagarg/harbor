const std = @import("std");

const source_text = @import("source_text.zig");

pub const CodePoint = u32;
pub const SourceCharacter = CodePoint;

// pub export const UTF16_MAX = source_text.UTF16_MAX;
// pub export const HIGH_SURROGATE_START = source_text.HIGH_SURROGATE_START;
// pub export const HIGH_SURROGATE_END = source_text.HIGH_SURROGATE_END;
// pub export const LOW_SURROGATE_START = source_text.LOW_SURROGATE_START;
// pub export const LOW_SURROGATE_END = source_text.LOW_SURROGATE_END;

pub export fn utf16_encode_cp(cp: CodePoint) source_text.String {
    return source_text.utf16_encode_cp(cp);
}

pub export fn cps_to_string(text: [*]CodePoint, len: usize) source_text.String {
    return source_text.cps_to_string(text, len) catch source_text.String{
        .data = &[_]u16{},
        .len = 0,
    };
}

pub export fn utf16_surrogate_pair_to_cp(high: u16, low: u16) CodePoint {
    return source_text.utf16_surrogate_pair_to_cp(high, low);
}

pub const CodePointAtResult = source_text.CodePointAtResult;

pub export fn code_point_at(text: source_text.String, position: usize) source_text.CodePointAtResult {
    return source_text.code_point_at(text, position);
}

pub export fn string_to_cps(text: source_text.String) [*]CodePoint {
    return source_text.string_to_cps(text) catch &[_]CodePoint{};
}
