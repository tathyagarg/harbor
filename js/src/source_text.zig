const std = @import("std");

const root = @import("root.zig");

pub const String = extern struct {
    data: [*]const u16,
    len: usize,
};

pub const UTF16_MAX = 0x10FFFF;

pub const HIGH_SURROGATE_START = 0xD800;
pub const HIGH_SURROGATE_END = 0xDBFF;

pub const LOW_SURROGATE_START = 0xDC00;
pub const LOW_SURROGATE_END = 0xDFFF;

pub fn utf16_encode_cp(cp: root.CodePoint) String {
    std.debug.assert(cp <= UTF16_MAX);

    if (cp <= 0xFFFF) {
        return String{
            .data = &[_]u16{@intCast(cp)},
            .len = 1,
        };
    } else {
        const high_surrogate: u16 = @intCast(((cp - 0x10000) >> 10) + 0xD800);
        const low_surrogate: u16 = @intCast(((cp - 0x10000) & 0x3FF) + 0xDC00);
        return String{
            .data = &[_]u16{ high_surrogate, low_surrogate },
            .len = 2,
        };
    }
}

pub fn cps_to_string(text: [*]root.CodePoint, len: usize) !String {
    var result: std.ArrayList(u16) = .empty;
    defer result.deinit(std.heap.page_allocator);

    for (text, 0..len) |cp, _| {
        const encoded = utf16_encode_cp(cp);
        _ = try result.appendSlice(std.heap.page_allocator, encoded.data[0..encoded.len]);
    }

    return String{
        .data = (try result.toOwnedSlice(std.heap.page_allocator)).ptr,
        .len = result.items.len,
    };
}

pub fn utf16_surrogate_pair_to_cp(high: u16, low: u16) root.CodePoint {
    std.debug.assert(high >= 0xD800 and high <= 0xDBFF);
    std.debug.assert(low >= 0xDC00 and low <= 0xDFFF);

    const high_part: u21 = @intCast(high - 0xD800);
    const low_part: u21 = @intCast(low - 0xDC00);
    return (high_part * 0x400) + low_part + 0x10000;
}

pub const CodePointAtResult = extern struct {
    code_point: root.CodePoint,
    code_unit_count: usize,
    is_unpaired_surrogate: bool,
};

pub fn is_leading_surrogate(unit: u16) bool {
    return unit >= HIGH_SURROGATE_START and unit <= HIGH_SURROGATE_END;
}

pub fn is_trailing_surrogate(unit: u16) bool {
    return unit >= LOW_SURROGATE_START and unit <= LOW_SURROGATE_END;
}

pub fn code_point_at(text: String, position: usize) CodePointAtResult {
    const size = text.len;

    std.debug.assert(position < size);

    const first_unit: u16 = text.data[position];
    var cp: root.CodePoint = @intCast(first_unit);

    if (!is_leading_surrogate(first_unit) and !is_trailing_surrogate(first_unit)) {
        return CodePointAtResult{
            .code_point = cp,
            .code_unit_count = 1,
            .is_unpaired_surrogate = false,
        };
    }

    if (is_trailing_surrogate(first_unit) or position + 1 == size) {
        return CodePointAtResult{
            .code_point = cp,
            .code_unit_count = 1,
            .is_unpaired_surrogate = true,
        };
    }

    const second_unit = text.data[position + 1];

    if (!is_trailing_surrogate(second_unit)) {
        return CodePointAtResult{
            .code_point = cp,
            .code_unit_count = 1,
            .is_unpaired_surrogate = true,
        };
    }

    cp = utf16_surrogate_pair_to_cp(first_unit, second_unit);
    return CodePointAtResult{
        .code_point = cp,
        .code_unit_count = 2,
        .is_unpaired_surrogate = false,
    };
}

pub fn string_to_cps(text: String) ![*]root.CodePoint {
    var code_points: std.ArrayList(root.CodePoint) = .empty;
    defer code_points.deinit(std.heap.page_allocator);

    var i: usize = 0;
    while (i < text.len) {
        const result = code_point_at(text, i);
        _ = try code_points.append(std.heap.page_allocator, result.code_point);
        i += result.code_unit_count;
    }

    return (try code_points.toOwnedSlice(std.heap.page_allocator)).ptr;
}
