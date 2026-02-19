const std = @import("std");

const _text = @import("../text.zig");

const testing = @import("../../testing.zig");

const TokenSeq = _text.TokenSeq;
const Token = _text.Token;

const CommonTokenData = _text.CommonTokenData;
const IdentifierNameData = _text.IdentifierNameData;

const expr = @import("expressions.zig");

pub const Parser = struct {
    tokens: TokenSeq,
    curr: usize,

    arena: std.heap.ArenaAllocator,
    allocator: std.mem.Allocator,
};

pub fn init(tokens: TokenSeq) Parser {
    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };

    parser.allocator = parser.arena.allocator();

    return parser;
}

pub fn deinit(parser: *Parser) void {
    parser.arena.deinit();
}

pub fn next(parser: *Parser) ?Token {
    if (parser.curr >= parser.tokens.len) {
        return null;
    }
    const token = parser.tokens.data[parser.curr];
    parser.curr += 1;
    return token;
}

pub fn peek(parser: *Parser) ?Token {
    if (parser.curr >= parser.tokens.len) {
        return null;
    }
    return parser.tokens.data[parser.curr];
}

pub fn expect(parser: *Parser, expected: Token) !void {
    const token = peek(parser) orelse return error.UnexpectedEndOfTokens;

    if (token.kind != expected.kind) {
        return error.UnexpectedToken;
    }

    if (token.kind == .CommonToken) {
        const token_data: *CommonTokenData = @ptrFromInt(token.data);
        const expected_data: *CommonTokenData = @ptrFromInt(expected.data);

        if (token_data.common_token_kind != expected_data.common_token_kind) {
            return error.UnexpectedToken;
        }

        switch (token_data.common_token_kind) {
            .IdentifierName => {
                const token_idtfr_data: *IdentifierNameData = @ptrFromInt(token_data.data);
                const expected_idtfr_data: *IdentifierNameData = @ptrFromInt(expected_data.data);

                if (!testing.are_equal_strings(token_idtfr_data.name, expected_idtfr_data.name)) {
                    return error.UnexpectedToken;
                }
            },
            .NumericLiteral => {
                const token_num_data: *expr.NumericLiteralData = @ptrFromInt(token_data.data);
                const expected_num_data: *expr.NumericLiteralData = @ptrFromInt(expected_data.data);

                if (token_num_data.value != expected_num_data.value) {
                    return error.UnexpectedToken;
                }
            },
            .StringLiteral => {
                const token_str_data: *_text.StringLiteralData = @ptrFromInt(token_data.data);
                const expected_str_data: *_text.StringLiteralData = @ptrFromInt(expected_data.data);

                if (!testing.are_equal_strings(token_str_data.value, expected_str_data.value)) {
                    return error.UnexpectedToken;
                }
            },
            .Punctuator => {
                if (token_data.data != expected_data.data) {
                    return error.UnexpectedToken;
                }
            },
            else => {
                return error.UnexpectedToken;
            },
        }
    }
}

pub fn expect_skip_whitespace(parser: *Parser, expected: Token) !void {
    const initial_position = parser.curr;

    while (true) {
        const token = peek(parser) orelse return error.UnexpectedEndOfTokens;

        if (token.kind == .Whitespace or token.kind == .LineTerminator) {
            _ = next(parser);
            continue;
        }

        break;
    }

    expect(parser, expected) catch |err| {
        reset_to(parser, initial_position);
        return err;
    };
}

pub fn match(parser: *Parser, expected: Token) bool {
    const token = peek(parser);
    if (token == null) {
        return false;
    }
    if (token.? != expected) {
        return false;
    }
    next(parser);
    return true;
}

pub fn reset_to(parser: *Parser, position: usize) void {
    parser.curr = position;
}

pub fn parse_assignment_expression(parser: *Parser) !*expr.AssignmentExpression {
    const prim = try parse_primary_expression(parser);

    const assignment_expr = parser.allocator.create(expr.AssignmentExpression) catch return error.OutOfMemory;

    assignment_expr.* = expr.AssignmentExpression{
        .tag = expr.ASSIGNMENT_EXPR_PRIMARY,
        .data = .{
            .primary = prim,
        },
    };

    return assignment_expr;
}

pub fn parse_primary_expression(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory, UnexpectedToken }!*expr.PrimaryExpression {
    const token = peek(parser) orelse return error.UnexpectedEndOfTokens;

    const res = switch (token.kind) {
        .CommonToken => com: {
            const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

            break :com switch (common_token_data.common_token_kind) {
                .IdentifierName => idtfr: {
                    const identifier_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);

                    _ = next(parser);

                    const identifier_reference = parser.allocator.create(expr.IdentifierReference) catch return error.OutOfMemory;

                    identifier_reference.* = expr.IdentifierReference{
                        .tag = expr.IDENTIFIER_REF_IDENTIFIER,
                        .data = .{
                            .identifier = identifier_data,
                        },
                    };

                    break :idtfr expr.PrimaryExpression{
                        .tag = expr.PRIMARY_EXPR_IDENTIFIER,
                        .data = .{
                            .identifier = identifier_reference,
                        },
                    };
                },
                .NumericLiteral => num: {
                    const numeric_literal_data: *expr.NumericLiteralData = @ptrFromInt(common_token_data.data);

                    _ = next(parser);

                    const literal = parser.allocator.create(expr.Literal) catch return error.OutOfMemory;

                    literal.* = expr.Literal{
                        .tag = expr.LITERAL_NUMBER,
                        .data = .{
                            .number = numeric_literal_data,
                        },
                    };

                    break :num expr.PrimaryExpression{
                        .tag = expr.PRIMARY_EXPR_LITERAL,
                        .data = .{
                            .literal = literal,
                        },
                    };
                },
                .StringLiteral => str: {
                    const string_literal_data: *_text.StringLiteralData = @ptrFromInt(common_token_data.data);

                    _ = next(parser);

                    const literal = parser.allocator.create(expr.Literal) catch return error.OutOfMemory;

                    literal.* = expr.Literal{
                        .tag = expr.LITERAL_STRING,
                        .data = .{
                            .string = &string_literal_data.value,
                        },
                    };

                    break :str expr.PrimaryExpression{
                        .tag = expr.PRIMARY_EXPR_LITERAL,
                        .data = .{
                            .literal = literal,
                        },
                    };
                },
                .Punctuator => punc: {
                    const punctuator_kind: _text.PunctuatorKind = @enumFromInt(common_token_data.data);

                    const res = switch (punctuator_kind) {
                        .OpenParen => paren: {
                            _ = next(parser);

                            const inner = try parse_primary_expression(parser);

                            try expect_skip_whitespace(parser, Token{
                                .kind = .CommonToken,
                                .data = @intFromPtr(&CommonTokenData{
                                    .common_token_kind = .Punctuator,
                                    .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
                                }),
                            });

                            break :paren inner.*;
                        },
                        .OpenBracket => bracket: {
                            _ = next(parser);

                            var elems = std.ArrayList(expr.ArrayElement).empty;
                            defer elems.deinit(parser.allocator);

                            while (true) {
                                const next_token = peek(parser) orelse break;

                                if (next_token.kind == .Whitespace or next_token.kind == .LineTerminator) {
                                    _ = next(parser);
                                    continue;
                                }

                                if (next_token.kind != .CommonToken) {
                                    return error.UnexpectedToken;
                                }

                                const next_common_token_data: *CommonTokenData = @ptrFromInt(next_token.data);

                                if (next_common_token_data.common_token_kind == .Punctuator) {
                                    const next_punctuator_kind: _text.PunctuatorKind = @enumFromInt(next_common_token_data.data);

                                    if (next_punctuator_kind == .CloseBracket) {
                                        _ = next(parser);
                                        break;
                                    } else if (next_punctuator_kind == .Comma) {
                                        _ = next(parser);

                                        const ellision = parser.allocator.create(expr.ArrayElement) catch return error.OutOfMemory;

                                        ellision.* = expr.ArrayElement{
                                            .tag = expr.ARRAY_ELEMENT_ELLISION,
                                            .data = .{ .ellision = {} },
                                        };

                                        try elems.append(parser.allocator, ellision.*);
                                        continue;
                                    } else if (next_punctuator_kind == .Ellipsis) {
                                        _ = next(parser);

                                        const array_element = parser.allocator.create(expr.ArrayElement) catch return error.OutOfMemory;

                                        const rest_elem = try parse_assignment_expression(parser);

                                        array_element.* = expr.ArrayElement{
                                            .tag = expr.ARRAY_ELEMENT_SPREAD,
                                            .data = .{ .spread = rest_elem },
                                        };

                                        try elems.append(parser.allocator, array_element.*);

                                        try expect_skip_whitespace(parser, Token{
                                            .kind = .CommonToken,
                                            .data = @intFromPtr(&CommonTokenData{
                                                .common_token_kind = .Punctuator,
                                                .data = @intFromEnum(_text.PunctuatorKind.Comma),
                                            }),
                                        });
                                    } else {
                                        return error.UnexpectedToken;
                                    }
                                } else {
                                    const elem = try parse_assignment_expression(parser);

                                    const array_elem = parser.allocator.create(expr.ArrayElement) catch return error.OutOfMemory;

                                    array_elem.* = expr.ArrayElement{
                                        .tag = expr.ARRAY_ELEMENT_EXPR,
                                        .data = .{ .expression = elem },
                                    };

                                    try elems.append(parser.allocator, array_elem.*);

                                    expect_skip_whitespace(parser, Token{
                                        .kind = .CommonToken,
                                        .data = @intFromPtr(&CommonTokenData{
                                            .common_token_kind = .Punctuator,
                                            .data = @intFromEnum(_text.PunctuatorKind.Comma),
                                        }),
                                    }) catch try expect_skip_whitespace(parser, Token{
                                        .kind = .CommonToken,
                                        .data = @intFromPtr(&CommonTokenData{
                                            .common_token_kind = .Punctuator,
                                            .data = @intFromEnum(_text.PunctuatorKind.CloseBracket),
                                        }),
                                    });

                                    _ = next(parser);
                                }
                            }

                            const array_literal = parser.allocator.create(expr.ArrayLiteral) catch return error.OutOfMemory;

                            const seq = parser.allocator.create(_text.Seq(expr.ArrayElement)) catch return error.OutOfMemory;

                            const slice = elems.toOwnedSlice(parser.allocator) catch return error.OutOfMemory;

                            seq.* = .{
                                .data = slice.ptr,
                                .len = slice.len,
                            };

                            array_literal.* = expr.ArrayLiteral{
                                .elements = seq.*,
                            };

                            break :bracket expr.PrimaryExpression{
                                .tag = expr.PRIMARY_EXPR_ARRAY,
                                .data = .{
                                    .array = array_literal,
                                },
                            };
                        },
                        else => {
                            return error.UnexpectedToken;
                        },
                    };

                    break :punc res;
                },
                else => {
                    return error.UnexpectedToken;
                },
            };
        },
        else => {
            return error.UnexpectedToken;
        },
    };

    const primary_expr = parser.allocator.create(expr.PrimaryExpression) catch return error.OutOfMemory;
    primary_expr.* = res;

    return primary_expr;
}

test "parse primary expr identifier" {
    const text = "myVar";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = [_]Token{
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .IdentifierName,
                .data = @intFromPtr(&IdentifierNameData{
                    .name = str,
                }),
            }),
        },
    };

    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();

    var parser = Parser{
        .tokens = TokenSeq{
            .data = &tokens,
            .len = 1,
        },
        .curr = 0,
        .arena = arena,
        .allocator = arena.allocator(),
    };

    const result = (try parse_primary_expression(&parser)).*;

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_IDENTIFIER);
    std.debug.assert(result.data.identifier.tag == expr.IDENTIFIER_REF_IDENTIFIER);
    std.debug.assert(testing.are_equal_strings(result.data.identifier.data.identifier.name, str));

    _text.free_string(str);
    deinit(&parser);
}

test "parse primary expr numeric literal" {
    const tokens = [_]Token{
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .NumericLiteral,
                .data = @intFromPtr(&expr.NumericLiteralData{
                    .value = 42.0,
                    .is_bigint = false,
                    .number_system = .Decimal,
                }),
            }),
        },
    };

    var parser = init(TokenSeq{
        .data = &tokens,
        .len = 1,
    });

    const result = (try parse_primary_expression(&parser)).*;

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.literal.data.number.value == 42.0);

    deinit(&parser);
}

test "parse primary expr string literal" {
    const text = "\"Hello, World!\"";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = [_]Token{
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .StringLiteral,
                .data = @intFromPtr(&_text.StringLiteralData{
                    .value = str,
                }),
            }),
        },
    };

    var parser = init(TokenSeq{
        .data = &tokens,
        .len = 1,
    });

    const result = (try parse_primary_expression(&parser)).*;

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.literal.tag == expr.LITERAL_STRING);
    std.debug.assert(testing.are_equal_strings(result.data.literal.data.string.*, str));

    _text.free_string(str);
    deinit(&parser);
}

test "parse primary expr array literal" {
    const tokens = [_]Token{
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .Punctuator,
                .data = @intFromEnum(_text.PunctuatorKind.OpenBracket),
            }),
        },
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .NumericLiteral,
                .data = @intFromPtr(&expr.NumericLiteralData{
                    .value = 1.0,
                    .is_bigint = false,
                    .number_system = .Decimal,
                }),
            }),
        },
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .Punctuator,
                .data = @intFromEnum(_text.PunctuatorKind.Comma),
            }),
        },
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .NumericLiteral,
                .data = @intFromPtr(&expr.NumericLiteralData{
                    .value = 2.0,
                    .is_bigint = false,
                    .number_system = .Decimal,
                }),
            }),
        },
        Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .Punctuator,
                .data = @intFromEnum(_text.PunctuatorKind.CloseBracket),
            }),
        },
    };

    var parser = Parser{
        .tokens = TokenSeq{
            .data = &tokens,
            .len = 5,
        },
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };

    parser.allocator = parser.arena.allocator();

    const result = (try parse_primary_expression(&parser)).*;

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_ARRAY);
    std.debug.assert(result.data.array.elements.len == 2);

    std.debug.assert(result.data.array.elements.data[0].tag == expr.ARRAY_ELEMENT_EXPR);
    std.debug.assert(result.data.array.elements.data[0].data.expression.tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.primary.data.literal.data.number.value == 1.0);

    std.debug.assert(result.data.array.elements.data[1].tag == expr.ARRAY_ELEMENT_EXPR);
    std.debug.assert(result.data.array.elements.data[1].data.expression.tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.primary.data.literal.data.number.value == 2.0);

    deinit(&parser);
}

test "parse primary expr from string" {
    const text = "[1, 2]";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_primary_expression(&parser)).*;

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_ARRAY);
    std.debug.assert(result.data.array.elements.len == 2);

    std.debug.assert(result.data.array.elements.data[0].tag == expr.ARRAY_ELEMENT_EXPR);
    std.debug.assert(result.data.array.elements.data[0].data.expression.tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.primary.data.literal.data.number.value == 1.0);

    std.debug.assert(result.data.array.elements.data[1].tag == expr.ARRAY_ELEMENT_EXPR);
    std.debug.assert(result.data.array.elements.data[1].data.expression.tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.primary.data.literal.data.number.value == 2.0);

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}
