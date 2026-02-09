const std = @import("std");

const root = @import("root.zig");

// js/mod.rs:ZigString
pub const String = extern struct {
    data: [*]const u16,
    len: usize,
};

// js/mod.rs:CodePointAtResult
pub const CodePointAtResult = extern struct {
    code_point: root.CodePoint,
    code_unit_count: usize,
    is_unpaired_surrogate: bool,
};

pub const CodePointSeq = extern struct {
    data: [*]const root.CodePoint,
    len: usize,
};

pub const GoalSymbol = enum(u8) {
    InputElementDiv,
    InputElementRegExp,
    InputElementTemplateTail,
    InputElementRegExpOrTemplateTail,
    InputElementHashbangOrRegExp,
};

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

pub const TokenKind = enum(u8) {
    Whitespace,
    LineTerminator,
    Comment,
    HashBangComment,
};

pub const Token = extern struct {
    kind: TokenKind,
    data: usize,
};

pub const TokenSeq = extern struct {
    data: [*]const Token,
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
        const buf = std.heap.page_allocator.alloc(u16, 1) catch {
            return String{
                .data = &[_]u16{},
                .len = 0,
            };
        };

        buf[0] = @intCast(cp);

        return String{
            .data = buf.ptr,
            .len = 1,
        };
    } else {
        const buf = std.heap.page_allocator.alloc(u16, 2) catch {
            return String{
                .data = &[_]u16{},
                .len = 0,
            };
        };

        const high_surrogate: u16 = @intCast(((cp - 0x10000) >> 10) + 0xD800);
        const low_surrogate: u16 = @intCast(((cp - 0x10000) & 0x3FF) + 0xDC00);

        buf[0] = high_surrogate;
        buf[1] = low_surrogate;

        return String{
            .data = buf.ptr,
            .len = 2,
        };
    }
}

pub fn cps_to_string(text: [*]root.CodePoint, len: usize) !String {
    var result: std.ArrayList(u16) = .empty;
    defer result.deinit(std.heap.page_allocator);

    for (text[0..len]) |cp| {
        const encoded = utf16_encode_cp(cp);
        _ = try result.appendSlice(std.heap.page_allocator, encoded.data[0..encoded.len]);
    }

    const owned = try result.toOwnedSlice(std.heap.page_allocator);

    return String{
        .data = owned.ptr,
        .len = owned.len,
    };
}

pub fn utf16_surrogate_pair_to_cp(high: u16, low: u16) root.CodePoint {
    std.debug.assert(high >= 0xD800 and high <= 0xDBFF);
    std.debug.assert(low >= 0xDC00 and low <= 0xDFFF);

    const high_part: u21 = @intCast(high - 0xD800);
    const low_part: u21 = @intCast(low - 0xDC00);
    return (high_part * 0x400) + low_part + 0x10000;
}

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

pub fn string_to_cps(text: String) !CodePointSeq {
    var code_points: std.ArrayList(root.CodePoint) = .empty;
    defer code_points.deinit(std.heap.page_allocator);

    var i: usize = 0;
    while (i < text.len) {
        const result = code_point_at(text, i);
        _ = try code_points.append(std.heap.page_allocator, result.code_point);
        i += result.code_unit_count;
    }

    const owned = try code_points.toOwnedSlice(std.heap.page_allocator);

    return CodePointSeq{
        .data = owned.ptr,
        .len = owned.len,
    };
}

pub fn parse_text_string(text: String, goal: GoalSymbol) TokenSeq {
    const cps = string_to_cps(text) catch {
        std.debug.print("Failed to convert string to code points\n", .{});
        return TokenSeq{
            .data = &[_]Token{},
            .len = 0,
        };
    };

    return parse_text_cps(cps, goal) catch {
        std.debug.print("Failed to parse text code points\n", .{});
        return TokenSeq{
            .data = &[_]Token{},
            .len = 0,
        };
    };
}

pub fn parse_text_cps(text: CodePointSeq, goal: GoalSymbol) !TokenSeq {
    return parse_goal(text, goal);
}

pub fn parse_goal(text: CodePointSeq, goal: GoalSymbol) !TokenSeq {
    return switch (goal) {
        .InputElementHashbangOrRegExp => parse_input_element_hashbang_or_regexp(text),
        else => {
            std.debug.print("Goal symbol not implemented\n", .{});
            return TokenSeq{
                .data = &[_]Token{},
                .len = 0,
            };
        },
    };
}

pub fn parse_input_element_hashbang_or_regexp(text: CodePointSeq) !TokenSeq {
    var i: usize = 0;

    var tokens: std.ArrayList(Token) = .empty;
    defer tokens.deinit(std.heap.page_allocator);

    while (i < text.len) {
        const cp = text.data[i];

        if (is_whitespace(cp)) {
            while (i < text.len and is_whitespace(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .Whitespace,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (is_line_terminator(cp)) {
            while (i < text.len and is_line_terminator(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .LineTerminator,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (cp == 0x002F and (i + 1 < text.len) and text.data[i + 1] == 0x002A) {
            i += 2;

            while (i + 1 < text.len) {
                if (text.data[i] == 0x002A and text.data[i + 1] == 0x002F) {
                    i += 2;
                    break;
                }
                i += 1;
            }

            tokens.append(std.heap.page_allocator, Token{
                .kind = .Comment,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (cp == 0x002F and (i + 1 < text.len) and text.data[i + 1] == 0x002F) {
            i += 2;

            while (i < text.len and !is_line_terminator(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .Comment,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (cp == 0x021 and (i + 1 < text.len) and text.data[i + 1] == 0x002F) {
            i += 2;

            while (i < text.len and !is_line_terminator(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .HashBangComment,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else {
            // For simplicity, we stop parsing on the first non-whitespace, non-comment character.
            break;
        }
    }

    const owned = try tokens.toOwnedSlice(std.heap.page_allocator);

    return TokenSeq{
        .data = owned.ptr,
        .len = owned.len,
    };
}

pub fn is_whitespace(cp: root.CodePoint) bool {
    return std.mem.indexOf(root.CodePoint, &WHITESPACE_CHARS, &[_]root.CodePoint{cp}) != null;
}

pub fn is_line_terminator(cp: root.CodePoint) bool {
    return std.mem.indexOf(root.CodePoint, &LINE_TERMINATOR_CHARS, &[_]root.CodePoint{cp}) != null;
}

fn u8_array_to_string(text: [*]u8, len: usize) String {
    const buf = std.heap.page_allocator.alloc(u16, len) catch {
        return String{
            .data = &[_]u16{},
            .len = 0,
        };
    };

    for (text, 0..len) |b, i| {
        buf[i] = @intCast(b);
    }

    return String{
        .data = buf.ptr,
        .len = len,
    };
}

test "parse input element hashbang or regexp #1" {
    const text =
        \\!// This is a hashbang comment
        \\/* This is a block comment 
        \\that spans multiple lines */
        \\// This is a line comment
        \\    // This is a whitespace followed by a comment
    ;

    const string = u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = parse_text_string(string, .InputElementHashbangOrRegExp);

    const expected_kinds = [_]TokenKind{
        .HashBangComment,
        .LineTerminator,
        .Comment,
        .LineTerminator,
        .Comment,
        .LineTerminator,
        .Whitespace,
        .Comment,
    };

    for (tokens.data[0..tokens.len], 0..) |token, i| {
        std.debug.assert(token.kind == expected_kinds[i]);
    }
}
