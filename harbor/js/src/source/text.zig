const std = @import("std");

const root = @import("../root.zig");
const unicode = @import("unicode.zig");
const numeral = @import("numeral.zig");

const testing = @import("../testing.zig");

const OPEN_BRACE = 0x007B;
const CLOSE_BRACE = 0x007D;
const OPEN_PAREN = 0x0028;
const CLOSE_PAREN = 0x0029;
const OPEN_BRACKET = 0x005B;
const CLOSE_BRACKET = 0x005D;
const PERIOD = 0x002E;
const SEMICOLON = 0x003B;
const COMMA = 0x002C;
const LESS_THAN = 0x003C;
const GREATER_THAN = 0x003E;
const EXCLAMATION_MARK = 0x0021;
const PLUS_SIGN = 0x002B;
const HYPHEN_MINUS = 0x002D;
const ASTERISK = 0x002A;
const SOLIDUS = 0x002F;
const PERCENT_SIGN = 0x0025;
const AMPERSAND = 0x0026;
const VERTICAL_LINE = 0x007C;
const CIRCUMFLEX_ACCENT = 0x005E;
const TILDE = 0x007E;
const QUESTION_MARK = 0x003F;
const COLON = 0x003A;
const EQUALS_SIGN = 0x003D;
const HASH_SIGN = 0x0023;

const SINGLE_QUOTE = 0x0027;
const DOUBLE_QUOTE = 0x0022;
const BACKSLASH = 0x005C;

const ZERO = 0x0030;
const UNDERSCORE = 0x005F;

pub fn Seq(comptime T: type) type {
    return extern struct {
        data: [*]const T,
        len: usize,
    };
}

// js/mod.rs:ZigString
pub const String = Seq(u16);

// js/mod.rs:CodePointAtResult
pub const CodePointAtResult = extern struct {
    code_point: root.CodePoint,
    code_unit_count: usize,
    is_unpaired_surrogate: bool,
};

pub const CodePointSeq = Seq(root.CodePoint);

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

pub const NumericLiteralNumberSystem = enum(u8) {
    Decimal,
    Binary,
    Octal,
    Hexadecimal,
};

pub const NumericLiteralData = extern struct {
    value: f64,
    is_bigint: bool,
    number_system: NumericLiteralNumberSystem,
};

pub const StringLiteralData = extern struct {
    value: String,
};

pub const CommonTokenData = extern struct {
    common_token_kind: CommonTokenKind,
    data: usize,
};

pub const Token = extern struct {
    kind: TokenKind,
    data: usize,
};

pub const TokenSeq = Seq(Token);

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

    return parse_text_cps(cps, goal);
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
        } else if (cp == SOLIDUS and (i + 1 < text.len) and text.data[i + 1] == ASTERISK) {
            i += 2;

            while (i + 1 < text.len) {
                if (text.data[i] == ASTERISK and text.data[i + 1] == SOLIDUS) {
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
        } else if (cp == SOLIDUS and (i + 1 < text.len) and text.data[i + 1] == SOLIDUS) {
            i += 2;

            while (i < text.len and !unicode.is_line_terminator(text.data[i])) : (i += 1) {}

            tokens.append(std.heap.page_allocator, Token{
                .kind = .Comment,
                .data = 0,
            }) catch {
                std.debug.print("Failed to append token\n", .{});
                return error.Generic;
            };
        } else if (cp == HASH_SIGN and (i + 1 < text.len) and text.data[i + 1] == EXCLAMATION_MARK) {
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
            } else if (cp == HASH_SIGN) {
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
            } else if (numeral.match_numeric_literal(text, &i, cp)) |token| {
                tokens.append(std.heap.page_allocator, token) catch {
                    std.debug.print("Failed to append token\n", .{});
                    return error.Generic;
                };
            } else if (match_string_literal(text, &i, cp)) |token| {
                tokens.append(std.heap.page_allocator, token) catch {
                    std.debug.print("Failed to append token\n", .{});
                    return error.Generic;
                };
            } else {
                // std.debug.print("Unexpected character: {x}\n", .{cp});
                std.debug.print("test: {any}", .{numeral.match_numeric_literal(text, &i, cp)});
                i += 1;
            }
        }
    }

    const owned = try tokens.toOwnedSlice(std.heap.page_allocator);

    return TokenSeq{
        .data = owned.ptr,
        .len = owned.len,
    };
}

fn match_string_literal(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?Token {
    if (cp == DOUBLE_QUOTE or cp == SINGLE_QUOTE) {
        const quote = cp;
        var chars: std.ArrayList(root.CodePoint) = .empty;
        defer chars.deinit(std.heap.page_allocator);

        i.* += 1;

        while (i.* < text.len and text.data[i.*] != quote) {
            if (text.data[i.*] == BACKSLASH) {
                const next = if (i.* + 1 < text.len) text.data[i.* + 1] else 0;
                const next_to_next = if (i.* + 2 < text.len) text.data[i.* + 2] else 0;

                if (unicode.is_character_escape_sequence(cp)) {
                    const char = unicode.get_corresponding_character_escape(next);
                    _ = chars.append(std.heap.page_allocator, char.?) catch {
                        return null;
                    };
                } else if (next == ZERO and !unicode.is_decimal_digit(next_to_next)) {
                    _ = chars.append(std.heap.page_allocator, ZERO) catch {
                        return null;
                    };
                } else {
                    // For now, we will just treat unrecognized escape sequences as the character itself
                    _ = chars.append(std.heap.page_allocator, next) catch {
                        return null;
                    };
                }

                i.* += 2;
            } else {
                if (text.data[i.*] != quote and text.data[i.*] != BACKSLASH) {
                    _ = chars.append(std.heap.page_allocator, text.data[i.*]) catch {
                        return null;
                    };

                    i.* += 1;
                }
            }
        }

        if (i.* < text.len and text.data[i.*] == quote) {
            i.* += 1; // Skip closing quote

            const string_data = std.heap.page_allocator.create(StringLiteralData) catch {
                std.debug.print("Failed to create string literal data\n", .{});
                return null;
            };

            string_data.* = StringLiteralData{
                .value = cps_to_string(chars.items.ptr, chars.items.len) catch {
                    std.debug.print("Failed to convert string literal chars to string\n", .{});
                    return null;
                },
            };

            const common_token_data = std.heap.page_allocator.create(CommonTokenData) catch {
                std.debug.print("Failed to create common token data\n", .{});
                return null;
            };

            common_token_data.* = CommonTokenData{
                .common_token_kind = .StringLiteral,
                .data = @intFromPtr(string_data),
            };

            return Token{
                .kind = .CommonToken,
                .data = @intFromPtr(common_token_data),
            };
        } else {
            std.debug.print("Unterminated string literal\n", .{});
            return null;
        }
    }

    return null;
}

test "match string literal #1" {
    const str = "\"Hello, world!\"";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = string_to_cps(text) catch {
        std.debug.print("Failed to convert string to code points\n", .{});
        return;
    };

    var i: usize = 0;
    const token = match_string_literal(cps, &i, text.data[i]);
    std.debug.assert(token != null);
    const token_data: *CommonTokenData = @ptrFromInt(token.?.data);
    const string_data: *StringLiteralData = @ptrFromInt(token_data.data);
    std.debug.assert(string_data.value.len == 13);

    for (string_data.value.data[0..string_data.value.len], 0..) |unit, idx| {
        std.debug.assert(unit == "Hello, world!"[idx]);
    }

    free_code_point_seq(cps);
    free_string(text);
}

test "match string literal #2" {
    const str = "\"Hello, \\\"world\\\"!\"";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = string_to_cps(text) catch {
        std.debug.print("Failed to convert string to code points\n", .{});
        return;
    };

    var i: usize = 0;
    const token = match_string_literal(cps, &i, text.data[i]);
    std.debug.assert(token != null);
    const token_data: *CommonTokenData = @ptrFromInt(token.?.data);
    const string_data: *StringLiteralData = @ptrFromInt(token_data.data);
    std.debug.assert(string_data.value.len == 15);

    for (string_data.value.data[0..string_data.value.len], 0..) |unit, idx| {
        std.debug.assert(unit == "Hello, \"world\"!"[idx]);
    }

    free_code_point_seq(cps);
    free_string(text);
}

test "single quote string literal" {
    const str = "'Hello, world!'";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = string_to_cps(text) catch {
        std.debug.print("Failed to convert string to code points\n", .{});
        return;
    };

    var i: usize = 0;
    const token = match_string_literal(cps, &i, text.data[i]);
    std.debug.assert(token != null);
    const token_data: *CommonTokenData = @ptrFromInt(token.?.data);
    const string_data: *StringLiteralData = @ptrFromInt(token_data.data);
    std.debug.assert(string_data.value.len == 13);

    for (string_data.value.data[0..string_data.value.len], 0..) |unit, idx| {
        std.debug.assert(unit == "Hello, world!"[idx]);
    }

    free_code_point_seq(cps);
    free_string(text);
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

    if (cp == OPEN_BRACE) {
        i.* += 1;
        kind = .OpenBrace;
    } else if (cp == CLOSE_BRACE) {
        i.* += 1;
        kind = .CloseBrace;
    } else if (cp == OPEN_PAREN) {
        i.* += 1;
        kind = .OpenParen;
    } else if (cp == CLOSE_PAREN) {
        i.* += 1;
        kind = .CloseParen;
    } else if (cp == OPEN_BRACKET) {
        i.* += 1;
        kind = .OpenBracket;
    } else if (cp == CLOSE_BRACKET) {
        i.* += 1;
        kind = .CloseBracket;
    } else if (cp == PERIOD) {
        i.* += 1;

        if (next == PERIOD and next_to_next == PERIOD) {
            i.* += 2;
            kind = .Ellipsis;
        } else {
            kind = .Period;
        }
    } else if (cp == SEMICOLON) {
        i.* += 1;
        kind = .Semicolon;
    } else if (cp == COMMA) {
        i.* += 1;
        kind = .Comma;
    } else if (cp == LESS_THAN) {
        i.* += 1;

        if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .LessThanEqual;
        } else if (next == LESS_THAN) {
            i.* += 1;

            if (next_to_next == EQUALS_SIGN) {
                i.* += 1;
                kind = .LeftShiftAssign;
            } else {
                kind = .LeftShift;
            }
        } else {
            kind = .LessThan;
        }
    } else if (cp == GREATER_THAN) {
        i.* += 1;

        if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .GreaterThanEqual;
        } else if (next == GREATER_THAN) {
            i.* += 1;

            if (next_to_next == EQUALS_SIGN) {
                i.* += 1;
                kind = .RightShiftAssign;
            } else if (next_to_next == GREATER_THAN) {
                i.* += 1;

                if (next_to_next_to_next == EQUALS_SIGN) {
                    i.* += 1;
                    kind = .UnsignedRightShiftAssign;
                } else {
                    kind = .UnsignedRightShift;
                }
            } else {
                kind = .RightShift;
            }
        } else {
            kind = .GreaterThan;
        }
    } else if (cp == EXCLAMATION_MARK) {
        i.* += 1;

        if (next == EQUALS_SIGN and next_to_next == EQUALS_SIGN) {
            i.* += 2;
            kind = .StrictNotEquals;
        } else if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .NotEquals;
        } else {
            kind = .Not;
        }
    } else if (cp == PLUS_SIGN) {
        i.* += 1;

        if (next == PLUS_SIGN) {
            i.* += 1;
            kind = .Increment;
        } else if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .PlusAssign;
        } else {
            kind = .Plus;
        }
    } else if (cp == HYPHEN_MINUS) {
        i.* += 1;

        if (next == HYPHEN_MINUS) {
            i.* += 1;
            kind = .Decrement;
        } else if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .MinusAssign;
        } else {
            kind = .Minus;
        }
    } else if (cp == ASTERISK) {
        i.* += 1;

        if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .AsteriskAssign;
        } else if (next == ASTERISK) {
            i.* += 1;

            if (next_to_next == EQUALS_SIGN) {
                i.* += 1;
                kind = .ExponentiationAssign;
            } else {
                kind = .Exponentiation;
            }
        } else {
            kind = .Asterisk;
        }
    } else if (cp == SOLIDUS) {
        i.* += 1;

        if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .SlashAssign;
        } else {
            kind = .Slash;
        }
    } else if (cp == PERCENT_SIGN) {
        i.* += 1;

        if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .PercentAssign;
        } else {
            kind = .Percent;
        }
    } else if (cp == AMPERSAND) {
        i.* += 1;

        if (next == AMPERSAND) {
            i.* += 1;

            if (next_to_next == EQUALS_SIGN) {
                i.* += 1;
                kind = .LogicalAndAssign;
            } else {
                kind = .LogicalAnd;
            }
        } else if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .BitwiseAndAssign;
        } else {
            kind = .BitwiseAnd;
        }
    } else if (cp == VERTICAL_LINE) {
        i.* += 1;

        if (next == VERTICAL_LINE) {
            i.* += 1;

            if (next_to_next == EQUALS_SIGN) {
                i.* += 1;
                kind = .LogicalOrAssign;
            } else {
                kind = .LogicalOr;
            }
        } else if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .BitwiseOrAssign;
        } else {
            kind = .BitwiseOr;
        }
    } else if (cp == CIRCUMFLEX_ACCENT) {
        i.* += 1;

        if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .BitwiseXorAssign;
        } else {
            kind = .BitwiseXor;
        }
    } else if (cp == TILDE) {
        i.* += 1;
        kind = .BitwiseNot;
    } else if (cp == QUESTION_MARK) {
        i.* += 1;

        if (next == QUESTION_MARK) {
            i.* += 1;

            if (next_to_next == EQUALS_SIGN) {
                i.* += 1;
                kind = .NullishCoalescingAssign;
            } else {
                kind = .NullishCoalescing;
            }
        } else if (next == PERIOD and !unicode.is_decimal_digit(next_to_next)) {
            i.* += 1;
            kind = .OptionalChaining;
        } else {
            kind = .QuestionMark;
        }
    } else if (cp == COLON) {
        i.* += 1;
        kind = .Colon;
    } else if (cp == EQUALS_SIGN) {
        i.* += 1;

        if (next == GREATER_THAN) {
            i.* += 1;
            kind = .FunctionArrow;
        } else if (next == EQUALS_SIGN and next_to_next == EQUALS_SIGN) {
            i.* += 2;
            kind = .StrictEquals;
        } else if (next == EQUALS_SIGN) {
            i.* += 1;
            kind = .Equals;
        } else {
            kind = .Assign;
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

pub fn display_code_point_seq(seq: CodePointSeq) void {
    for (seq.data[0..seq.len]) |cp| {
        std.debug.print("{x} ", .{cp});
    }
    std.debug.print("\n", .{});
}

// =============================== TESTS ===============================

test "parse input element hashbang or regexp #1" {
    const text =
        \\#! This is a hashbang comment
        \\/* This is a block comment 
        \\that spans multiple lines */
        \\// This is a line comment
        \\    // This is a whitespace followed by a comment
    ;

    const string = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try parse_text_string(string, .InputElementHashbangOrRegExp);

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

    const string = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try parse_text_string(string, .InputElementHashbangOrRegExp);

    const expected_kinds = [_]TokenKind{
        .CommonToken,
    };

    const expected_ident = testing.u8_array_to_string(@ptrCast(@constCast("token")), 5);

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

    const string = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try parse_text_string(string, .InputElementHashbangOrRegExp);

    const expected_kinds = [_]TokenKind{
        .CommonToken,
    };

    const expected_ident = testing.u8_array_to_string(@ptrCast(@constCast("privateIdentifier")), 17);

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

    const string = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try parse_text_string(string, .InputElementHashbangOrRegExp);

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
