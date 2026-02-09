const std = @import("std");

const root = @import("root.zig");
const unicode = @import("unicode.zig");

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

pub const TokenKind = enum(u8) {
    Whitespace,
    LineTerminator,
    Comment,
    HashBangComment,
    CommonToken,
};

pub const CommonTokenKind = enum(u8) {
    Identifier,
    NumericLiteral,
    StringLiteral,
    Punctuator,
    Template,
};

pub const IdentifierData = extern struct {
    name: String,
};

pub const CommonTokenData = struct {
    common_token_kind: CommonTokenKind,
    data: usize,
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

pub fn code_point_at(text: String, position: usize) CodePointAtResult {
    const size = text.len;

    std.debug.assert(position < size);

    const first_unit: u16 = text.data[position];
    var cp: root.CodePoint = @intCast(first_unit);

    if (!unicode.is_leading_surrogate(first_unit) and !unicode.is_trailing_surrogate(first_unit)) {
        return CodePointAtResult{
            .code_point = cp,
            .code_unit_count = 1,
            .is_unpaired_surrogate = false,
        };
    }

    if (unicode.is_trailing_surrogate(first_unit) or position + 1 == size) {
        return CodePointAtResult{
            .code_point = cp,
            .code_unit_count = 1,
            .is_unpaired_surrogate = true,
        };
    }

    const second_unit = text.data[position + 1];

    if (!unicode.is_trailing_surrogate(second_unit)) {
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

        if (unicode.is_whitespace(cp)) {
            while (i < text.len and unicode.is_whitespace(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .Whitespace,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (unicode.is_line_terminator(cp)) {
            while (i < text.len and unicode.is_line_terminator(text.data[i])) : (i += 1) {}

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

            while (i < text.len and !unicode.is_line_terminator(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .Comment,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (cp == 0x021 and (i + 1 < text.len) and text.data[i + 1] == 0x002F) {
            i += 2;

            while (i < text.len and !unicode.is_line_terminator(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .HashBangComment,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else {
            const unicode_res = unicode.is_unicode_escape_sequence(text.data[i..], text.len - i);

            if (unicode.is_identifier_start(cp) or unicode_res != null) {
                var chars: std.ArrayList(root.CodePoint) = .empty;
                defer chars.deinit(std.heap.page_allocator);

                while (i < text.len and
                    (unicode.is_identifier_part(text.data[i]) or
                        unicode.is_unicode_escape_sequence(text.data[i..], text.len - i) != null))
                {
                    if (unicode.is_unicode_escape_sequence(text.data[i..], text.len - i)) |count| {
                        _ = try chars.appendSlice(
                            std.heap.page_allocator,
                            text.data[i .. i + count],
                        );
                        i += count;
                    } else {
                        _ = try chars.append(std.heap.page_allocator, text.data[i]);

                        i += 1;
                    }
                }

                const ident_data = std.heap.page_allocator.create(IdentifierData) catch {
                    std.debug.print("Failed to create identifier data\n", .{});
                    return error.Generic;
                };

                ident_data.* = IdentifierData{
                    .name = try cps_to_string(chars.items.ptr, chars.items.len),
                };

                tokens.append(std.heap.page_allocator, Token{
                    .kind = .CommonToken,
                    .data = @intFromPtr(ident_data),
                }) catch {
                    std.debug.print("Failed to append token\n", .{});
                    return error.Generic;
                };
            }
        }
    }

    const owned = try tokens.toOwnedSlice(std.heap.page_allocator);

    return TokenSeq{
        .data = owned.ptr,
        .len = owned.len,
    };
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

// =============================== TESTS ===============================

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

test "parse input element hashbang or regexp #2" {
    const text =
        \\token
    ;

    const string = u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = parse_text_string(string, .InputElementHashbangOrRegExp);

    const expected_kinds = [_]TokenKind{
        .CommonToken,
    };

    const expected_ident = u8_array_to_string(@ptrCast(@constCast("token")), 5);

    for (tokens.data[0..tokens.len], 0..) |token, i| {
        std.debug.assert(token.kind == expected_kinds[i]);

        if (expected_kinds[i] == .CommonToken) {
            const ident_data: *IdentifierData = @ptrFromInt(token.data);

            std.debug.assert(ident_data.name.len == expected_ident.len);

            for (ident_data.name.data, 0..ident_data.name.len) |c, j| {
                std.debug.assert(c == expected_ident.data[j]);
            }
        }
    }
}
