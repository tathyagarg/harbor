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
    IdentifierName,
    PrivateIdentifier,
    Punctuator,
    NumericLiteral,
    StringLiteral,
    Template,
};

pub const PunctuatorKind = enum(u8) {
    OptionalChaining,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Period,
    Ellipsis,
    Semicolon,
    Comma,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Equals,
    NotEquals,
    StrictEquals,
    StrictNotEquals,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Exponentiation,
    Increment,
    Decrement,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Not,
    BitwiseNot,
    LogicalAnd,
    LogicalOr,
    NullishCoalescing,
    QuestionMark,
    Colon,
    Assign,
    PlusAssign,
    MinusAssign,
    AsteriskAssign,
    SlashAssign,
    PercentAssign,
    ExponentiationAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishCoalescingAssign,
    FunctionArrow,
};

pub const IdentifierNameData = extern struct {
    name: String,
};

pub const CommonTokenData = extern struct {
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

pub fn parse_text_string(text: String, goal: GoalSymbol) !TokenSeq {
    const cps = try string_to_cps(text);
    // catch {
    //     std.debug.print("Failed to convert string to code points\n", .{});
    //     return TokenSeq{
    //         .data = &[_]Token{},
    //         .len = 0,
    //     };
    // };

    return parse_text_cps(cps, goal);
    // catch {
    //     std.debug.print("Failed to parse text code points\n", .{});
    //     return TokenSeq{
    //         .data = &[_]Token{},
    //         .len = 0,
    //     };
    // };
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
            if (match_identifier_name(text, &i, cp)) |token| {
                tokens.append(std.heap.page_allocator, token) catch {
                    std.debug.print("Failed to append token\n", .{});
                    return error.Generic;
                };
            } else if (cp == 0x0023) {
                const next_cp = if (i + 1 < text.len) text.data[i + 1] else 0;

                i += 1;

                if (match_identifier_name(text, &i, next_cp)) |token| {
                    const common_token_data = std.heap.page_allocator.create(CommonTokenData) catch {
                        std.debug.print("Failed to create common token data\n", .{});
                        return error.Generic;
                    };

                    const token_data: *CommonTokenData = @ptrFromInt(token.data);

                    common_token_data.* = CommonTokenData{
                        .common_token_kind = .PrivateIdentifier,
                        .data = token_data.data,
                    };

                    const actual_token = Token{
                        .kind = .CommonToken,
                        .data = @intFromPtr(common_token_data),
                    };

                    tokens.append(std.heap.page_allocator, actual_token) catch {
                        std.debug.print("Failed to append token\n", .{});
                        return error.Generic;
                    };
                } else {
                    std.debug.print("Unexpected character after #: {x}\n", .{next_cp});
                }
            } else if (match_punctuator(text, &i, cp)) |token| {
                tokens.append(std.heap.page_allocator, token) catch {
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

fn match_identifier_name(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?Token {
    const unicode_res = unicode.is_unicode_escape_sequence(text.data[i.*..], text.len - i.*);

    if (unicode.is_identifier_start(cp) or unicode_res != null) {
        var chars: std.ArrayList(root.CodePoint) = .empty;
        defer chars.deinit(std.heap.page_allocator);

        while (i.* < text.len and
            (unicode.is_identifier_part(text.data[i.*]) or
                unicode.is_unicode_escape_sequence(text.data[i.*..], text.len - i.*) != null))
        {
            if (unicode.is_unicode_escape_sequence(text.data[i.*..], text.len - i.*)) |count| {
                _ = chars.appendSlice(
                    std.heap.page_allocator,
                    text.data[i.* .. i.* + count],
                ) catch {
                    return null;
                };
                i.* += count;
            } else {
                _ = chars.append(std.heap.page_allocator, text.data[i.*]) catch {
                    return null;
                };

                i.* += 1;
            }
        }

        const ident_data = std.heap.page_allocator.create(IdentifierNameData) catch {
            std.debug.print("Failed to create identifier data\n", .{});
            return null;
        };

        ident_data.* = IdentifierNameData{
            .name = cps_to_string(chars.items.ptr, chars.items.len) catch {
                std.debug.print("Failed to convert identifier chars to string\n", .{});
                return null;
            },
        };

        const common_token_data = std.heap.page_allocator.create(CommonTokenData) catch {
            std.debug.print("Failed to create common token data\n", .{});
            return null;
        };

        common_token_data.* = CommonTokenData{
            .common_token_kind = .IdentifierName,
            .data = @intFromPtr(ident_data),
        };

        return Token{
            .kind = .CommonToken,
            .data = @intFromPtr(common_token_data),
        };
    }

    return null;
}

fn match_punctuator(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?Token {
    const next = if (i.* + 1 < text.len) text.data[i.* + 1] else 0;
    const next_to_next = if (i.* + 2 < text.len) text.data[i.* + 2] else 0;
    const next_to_next_to_next = if (i.* + 3 < text.len) text.data[i.* + 3] else 0;

    const common_token_data = std.heap.page_allocator.create(CommonTokenData) catch {
        std.debug.print("Failed to create common token data\n", .{});
        return null;
    };

    common_token_data.* = CommonTokenData{
        .common_token_kind = .Punctuator,
        .data = 0,
    };

    var kind: ?PunctuatorKind = null;

    if (cp == 0x003F and next == 0x002E and !unicode.is_decimal_digit(next_to_next)) {
        i.* += 2;
        kind = PunctuatorKind.OptionalChaining;
    } else if (cp == 0x007B) {
        i.* += 1;
        kind = PunctuatorKind.OpenBrace;
    } else if (cp == 0x007D) {
        i.* += 1;
        kind = PunctuatorKind.CloseBrace;
    } else if (cp == 0x0028) {
        i.* += 1;
        kind = PunctuatorKind.OpenParen;
    } else if (cp == 0x0029) {
        i.* += 1;
        kind = PunctuatorKind.CloseParen;
    } else if (cp == 0x005B) {
        i.* += 1;
        kind = PunctuatorKind.OpenBracket;
    } else if (cp == 0x005D) {
        i.* += 1;
        kind = PunctuatorKind.CloseBracket;
    } else if (cp == 0x002E) {
        i.* += 1;

        if (next == 0x002E and next_to_next == 0x002E) {
            i.* += 2;
            kind = PunctuatorKind.Ellipsis;
        } else {
            kind = PunctuatorKind.Period;
        }
    } else if (cp == 0x003B) {
        i.* += 1;
        kind = PunctuatorKind.Semicolon;
    } else if (cp == 0x002C) {
        i.* += 1;
        kind = PunctuatorKind.Comma;
    } else if (cp == 0x003C) {
        i.* += 1;

        if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.LessThanEqual;
        } else if (next == 0x003C) {
            i.* += 1;

            if (next_to_next == 0x003D) {
                i.* += 1;
                kind = PunctuatorKind.LeftShiftAssign;
            } else {
                kind = PunctuatorKind.LeftShift;
            }
        } else {
            kind = PunctuatorKind.LessThan;
        }
    } else if (cp == 0x003E) {
        i.* += 1;

        if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.GreaterThanEqual;
        } else if (next == 0x003E) {
            i.* += 1;

            if (next_to_next == 0x003D) {
                i.* += 1;
                kind = PunctuatorKind.RightShiftAssign;
            } else if (next_to_next == 0x003E) {
                i.* += 1;

                if (next_to_next_to_next == 0x003D) {
                    i.* += 1;
                    kind = PunctuatorKind.UnsignedRightShiftAssign;
                } else {
                    kind = PunctuatorKind.UnsignedRightShift;
                }
            } else {
                kind = PunctuatorKind.RightShift;
            }
        } else {
            kind = PunctuatorKind.GreaterThan;
        }
    } else if (cp == 0x0021) {
        i.* += 1;

        if (next == 0x003D and next_to_next == 0x003D) {
            i.* += 2;
            kind = PunctuatorKind.StrictNotEquals;
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.NotEquals;
        } else {
            kind = PunctuatorKind.Not;
        }
    } else if (cp == 0x002B) {
        i.* += 1;

        if (next == 0x002B) {
            i.* += 1;
            kind = PunctuatorKind.Increment;
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.PlusAssign;
        } else {
            kind = PunctuatorKind.Plus;
        }
    } else if (cp == 0x002D) {
        i.* += 1;

        if (next == 0x002D) {
            i.* += 1;
            kind = PunctuatorKind.Decrement;
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.MinusAssign;
        } else {
            kind = PunctuatorKind.Minus;
        }
    } else if (cp == 0x002A) {
        i.* += 1;

        if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.AsteriskAssign;
        } else if (next == 0x002A) {
            i.* += 1;

            if (next_to_next == 0x003D) {
                i.* += 1;
                kind = PunctuatorKind.ExponentiationAssign;
            } else {
                kind = PunctuatorKind.Exponentiation;
            }
        } else {
            kind = PunctuatorKind.Asterisk;
        }
    } else if (cp == 0x002F) {
        i.* += 1;

        if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.SlashAssign;
        } else {
            kind = PunctuatorKind.Slash;
        }
    } else if (cp == 0x0025) {
        i.* += 1;

        if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.PercentAssign;
        } else {
            kind = PunctuatorKind.Percent;
        }
    } else if (cp == 0x0026) {
        i.* += 1;

        if (next == 0x0026) {
            i.* += 1;

            if (next_to_next == 0x003D) {
                i.* += 1;
                kind = PunctuatorKind.LogicalAndAssign;
            } else {
                kind = PunctuatorKind.LogicalAnd;
            }
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.BitwiseAndAssign;
        } else {
            kind = PunctuatorKind.BitwiseAnd;
        }
    } else if (cp == 0x007C) {
        i.* += 1;

        if (next == 0x007C) {
            i.* += 1;

            if (next_to_next == 0x003D) {
                i.* += 1;
                kind = PunctuatorKind.LogicalOrAssign;
            } else {
                kind = PunctuatorKind.LogicalOr;
            }
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.BitwiseOrAssign;
        } else {
            kind = PunctuatorKind.BitwiseOr;
        }
    } else if (cp == 0x005E) {
        i.* += 1;

        if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.BitwiseXorAssign;
        } else {
            kind = PunctuatorKind.BitwiseXor;
        }
    } else if (cp == 0x007E) {
        i.* += 1;
        kind = PunctuatorKind.BitwiseNot;
    } else if (cp == 0x0021) {
        i.* += 1;

        if (next == 0x003D and next_to_next == 0x003D) {
            i.* += 2;
            kind = PunctuatorKind.StrictNotEquals;
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.NotEquals;
        } else {
            kind = PunctuatorKind.Not;
        }
    } else if (cp == 0x003F) {
        i.* += 1;

        if (next == 0x003F) {
            i.* += 1;

            if (next_to_next == 0x003D) {
                i.* += 1;
                kind = PunctuatorKind.NullishCoalescingAssign;
            } else {
                kind = PunctuatorKind.NullishCoalescing;
            }
        } else {
            kind = PunctuatorKind.QuestionMark;
        }
    } else if (cp == 0x003A) {
        i.* += 1;
        kind = PunctuatorKind.Colon;
    } else if (cp == 0x003D) {
        i.* += 1;

        if (next == 0x003E) {
            i.* += 1;
            kind = PunctuatorKind.FunctionArrow;
        } else if (next == 0x003D and next_to_next == 0x003D) {
            i.* += 2;
            kind = PunctuatorKind.StrictEquals;
        } else if (next == 0x003D) {
            i.* += 1;
            kind = PunctuatorKind.Equals;
        } else {
            kind = PunctuatorKind.Assign;
        }
    }

    if (kind != null) {
        common_token_data.data = @intFromEnum(kind.?);
        return Token{
            .kind = .CommonToken,
            .data = @intFromPtr(common_token_data),
        };
    }

    return null;
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

pub fn free_string(str: String) void {
    std.heap.page_allocator.free(str.data[0..str.len]);
}

pub fn free_token_seq(token_seq: TokenSeq) void {
    for (token_seq.data[0..token_seq.len]) |token| {
        if (token.kind == .CommonToken) {
            const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

            if (common_token_data.common_token_kind == .IdentifierName or
                common_token_data.common_token_kind == .PrivateIdentifier)
            {
                const ident_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);
                free_string(ident_data.name);
                std.heap.page_allocator.destroy(ident_data);
            }

            std.heap.page_allocator.destroy(common_token_data);
        }
    }

    std.heap.page_allocator.free(token_seq.data[0..token_seq.len]);
}

pub fn free_code_point_seq(seq: CodePointSeq) void {
    std.heap.page_allocator.free(seq.data[0..seq.len]);
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
            const common_token: *CommonTokenData = @ptrFromInt(token.data);

            std.debug.assert(common_token.common_token_kind == .IdentifierName);

            const ident_data: *IdentifierNameData = @ptrFromInt(common_token.data);

            std.debug.assert(ident_data.name.len == expected_ident.len);

            for (ident_data.name.data, 0..ident_data.name.len) |c, j| {
                std.debug.assert(c == expected_ident.data[j]);
            }
        }
    }
}

test "parse input element hashbang or regexp #3" {
    const text =
        \\#privateIdentifier
    ;

    const string = u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = parse_text_string(string, .InputElementHashbangOrRegExp);

    const expected_kinds = [_]TokenKind{
        .CommonToken,
    };

    const expected_ident = u8_array_to_string(@ptrCast(@constCast("privateIdentifier")), 17);

    for (tokens.data[0..tokens.len], 0..) |token, i| {
        std.debug.assert(token.kind == expected_kinds[i]);

        if (expected_kinds[i] == .CommonToken) {
            const common_token: *CommonTokenData = @ptrFromInt(token.data);

            std.debug.assert(common_token.common_token_kind == .PrivateIdentifier);

            const ident_data: *IdentifierNameData = @ptrFromInt(common_token.data);

            std.debug.assert(ident_data.name.len == expected_ident.len);

            for (ident_data.name.data, 0..ident_data.name.len) |c, j| {
                std.debug.assert(c == expected_ident.data[j]);
            }
        }
    }
}

test "parse input element hashbang or regexp #4" {
    const text =
        \\?.?.? ??= ?.
    ;

    const string = u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = parse_text_string(string, .InputElementHashbangOrRegExp);

    const expected_kinds = [_]TokenKind{
        .CommonToken,
        .CommonToken,
        .CommonToken,
        .Whitespace,
        .CommonToken,
        .Whitespace,
        .CommonToken,
    };

    const expected_punctuators = [_]PunctuatorKind{
        .OptionalChaining,
        .OptionalChaining,
        .QuestionMark,
        .NullishCoalescingAssign,
        .OptionalChaining,
    };

    var seen_punctuators_index: usize = 0;

    for (tokens.data[0..tokens.len], 0..) |token, i| {
        std.debug.assert(token.kind == expected_kinds[i]);

        if (expected_kinds[i] == .CommonToken) {
            const common_token: *CommonTokenData = @ptrFromInt(token.data);

            std.debug.assert(common_token.common_token_kind == .Punctuator);

            const punct_data: PunctuatorKind = @enumFromInt(common_token.data);

            std.debug.assert(punct_data == expected_punctuators[seen_punctuators_index]);
            seen_punctuators_index += 1;
        }
    }
}
