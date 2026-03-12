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

pub fn skip_whitespace(parser: *Parser) void {
    while (true) {
        const token = peek(parser) orelse break;

        if (token.kind == .Whitespace or token.kind == .LineTerminator) {
            _ = next(parser);
            continue;
        }

        break;
    }
}

pub fn match(parser: *Parser, expected: Token) bool {
    const token = peek(parser);

    if (token == null) {
        return false;
    }

    if (!testing.are_equal_tokens(token.?, expected)) {
        return false;
    }

    _ = next(parser);
    return true;
}

pub fn reset_to(parser: *Parser, position: usize) void {
    parser.curr = position;
}

pub const MemberExpressionStarter = enum {
    Dot,
    OpenBracket,
};

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

fn token_is_member_expr_start(token: Token) ?MemberExpressionStarter {
    if (token.kind == .CommonToken) {
        const common_token_data: *CommonTokenData = @ptrFromInt(token.data);
        if (common_token_data.common_token_kind == .Punctuator) {
            const punctuator_kind: _text.PunctuatorKind = @enumFromInt(common_token_data.data);

            if (punctuator_kind == .Period) {
                return MemberExpressionStarter.Dot;
            } else if (punctuator_kind == .OpenBracket) {
                return MemberExpressionStarter.OpenBracket;
            }
        }
    }

    return null;
}

pub fn parse_member_expression(parser: *Parser) !*expr.MemberExpression {
    const prim = try parse_primary_expression(parser);

    const member_expr = parser.allocator.create(expr.MemberExpression) catch return error.OutOfMemory;
    member_expr.* = expr.MemberExpression{
        .tag = expr.MEMBER_EXPR_PRIMARY,
        .data = .{
            .primary = prim,
        },
    };

    while (peek(parser) != null) {
        const token = peek(parser).?;
        const member_expr_start = token_is_member_expr_start(token) orelse break;

        switch (member_expr_start) {
            .Dot => {
                _ = next(parser);

                const property_token = peek(parser) orelse return error.UnexpectedEndOfTokens;

                if (property_token.kind != .CommonToken) {
                    return error.UnexpectedToken;
                }

                const property_common_token_data: *CommonTokenData = @ptrFromInt(property_token.data);

                if (property_common_token_data.common_token_kind != .IdentifierName) {
                    return error.UnexpectedToken;
                }

                const property_identifier_data: *IdentifierNameData = @ptrFromInt(property_common_token_data.data);

                _ = next(parser);

                const member_expr_clone = parser.allocator.create(expr.MemberExpression) catch return error.OutOfMemory;
                member_expr_clone.* = member_expr.*;

                member_expr.* = expr.MemberExpression{
                    .tag = expr.MEMBER_EXPR_PROPERTY,
                    .data = .{
                        .property = .{
                            .object = member_expr_clone,
                            .property = property_identifier_data,
                        },
                    },
                };
            },
            .OpenBracket => {
                _ = next(parser);

                const property_expr = try parse_assignment_expression(parser);

                const property_arr = parser.allocator.create([1]expr.AssignmentExpression) catch return error.OutOfMemory;
                property_arr.* = [_]expr.AssignmentExpression{property_expr.*};

                // const property_arr = [_]expr.AssignmentExpression{property_expr.*};

                const property_expr_wrapper = parser.allocator.create(expr.Expression) catch return error.OutOfMemory;
                property_expr_wrapper.* = expr.Expression{
                    .data = &property_arr.*,
                    .len = 1,
                };

                try expect_skip_whitespace(parser, Token{
                    .kind = .CommonToken,
                    .data = @intFromPtr(&CommonTokenData{
                        .common_token_kind = .Punctuator,
                        .data = @intFromEnum(_text.PunctuatorKind.CloseBracket),
                    }),
                });

                const member_expr_clone = parser.allocator.create(expr.MemberExpression) catch return error.OutOfMemory;
                member_expr_clone.* = member_expr.*;

                member_expr.* = expr.MemberExpression{
                    .tag = expr.MEMBER_EXPR_MEMBER,
                    .data = .{
                        .member = .{
                            .object = member_expr_clone,
                            .expr = property_expr_wrapper,
                        },
                    },
                };
            },
        }
    }

    return member_expr;
}

pub fn parse_new_expression(parser: *Parser) !*expr.NewExpression {
    if (peek(parser) == null) {
        return error.UnexpectedEndOfTokens;
    }

    const token = peek(parser).?;

    if (token.kind == .CommonToken) {
        const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

        if (common_token_data.common_token_kind == .IdentifierName) {
            const identifier_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);

            if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("new")), 3))) {
                _ = next(parser);

                // skip whitespace after 'new' keyword
                _ = next(parser);

                const inner_expr = try parse_new_expression(parser);

                const new_expr = parser.allocator.create(expr.NewExpression) catch return error.OutOfMemory;
                new_expr.* = expr.NewExpression{
                    .tag = expr.NEW_EXPR_NEW,
                    .data = .{
                        .new = inner_expr,
                    },
                };

                return new_expr;
            }
        }
    }

    const member_expr = try parse_member_expression(parser);

    const new_expr = parser.allocator.create(expr.NewExpression) catch return error.OutOfMemory;
    new_expr.* = expr.NewExpression{
        .tag = expr.NEW_EXPR_MEMBER,
        .data = .{
            .member = member_expr,
        },
    };

    return new_expr;
}

pub fn parse_arguments(parser: *Parser) !*expr.Arguments {
    if (!match(parser, Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.OpenParen),
        }),
    })) {
        return error.UnexpectedToken;
    }

    var args = std.ArrayList(expr.AssignmentExpression).empty;
    defer args.deinit(parser.allocator);

    var is_spread = std.ArrayList(bool).empty;
    defer is_spread.deinit(parser.allocator);

    while (!match(parser, Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
        }),
    })) {
        const next_token = peek(parser) orelse return error.UnexpectedEndOfTokens;

        if (next_token.kind == .Whitespace or next_token.kind == .LineTerminator) {
            _ = next(parser);
            continue;
        }

        if (next_token.kind == .CommonToken) {
            const next_common_token_data: *CommonTokenData = @ptrFromInt(next_token.data);

            if (next_common_token_data.common_token_kind == .Punctuator) {
                const next_punctuator_kind: _text.PunctuatorKind = @enumFromInt(next_common_token_data.data);

                if (next_punctuator_kind == .Comma) {
                    _ = next(parser);
                    continue;
                }
            }

            if (next_common_token_data.common_token_kind == .Punctuator) {
                const next_punctuator_kind: _text.PunctuatorKind = @enumFromInt(next_common_token_data.data);

                if (next_punctuator_kind == .Ellipsis) {
                    _ = next(parser);
                    try is_spread.append(parser.allocator, true);
                } else {
                    try is_spread.append(parser.allocator, false);
                }
            } else {
                try is_spread.append(parser.allocator, false);
            }

            skip_whitespace(parser);

            const arg_expr = try parse_assignment_expression(parser);
            try args.append(parser.allocator, arg_expr.*);
        }
    }

    const args_slice = args.toOwnedSlice(parser.allocator) catch return error.OutOfMemory;
    const spread_slice = is_spread.toOwnedSlice(parser.allocator) catch return error.OutOfMemory;

    const args_seq = parser.allocator.create(_text.Seq(expr.AssignmentExpression)) catch return error.OutOfMemory;
    args_seq.* = .{
        .data = args_slice.ptr,
        .len = args_slice.len,
    };

    const arguments = parser.allocator.create(expr.Arguments) catch return error.OutOfMemory;
    arguments.* = expr.Arguments{
        .arguments = args_seq.*,
        .is_spread = spread_slice.ptr,
    };

    return arguments;
}

pub fn parse_call_expression(parser: *Parser) !*expr.CallExpression {
    const member_expr = try parse_member_expression(parser);
    skip_whitespace(parser);

    const arguments = try parse_arguments(parser);

    const call_expr = parser.allocator.create(expr.CallExpression) catch return error.OutOfMemory;
    call_expr.* = expr.CallExpression{
        .tag = expr.CALL_EXPR_COVER,
        .data = .{
            .cover = .{
                .callee = member_expr,
                .arguments = arguments,
            },
        },
    };

    // TODO: handle optional chaining call expressions and the other bs
    return call_expr;
}

pub fn parse_lhs_expression(parser: *Parser) !*expr.LeftHandSideExpression {
    const new_expr = try parse_new_expression(parser);

    const lhs = parser.allocator.create(expr.LeftHandSideExpression) catch return error.OutOfMemory;
    lhs.* = expr.LeftHandSideExpression{
        .tag = expr.LEFT_HAND_SIDE_EXPR_NEW,
        .data = .{
            .new = new_expr,
        },
    };

    skip_whitespace(parser);

    while (peek(parser) != null) {
        const token = peek(parser).?;

        if (token.kind == .CommonToken) {
            const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

            if (common_token_data.common_token_kind == .Punctuator) {
                const punctuator_kind: _text.PunctuatorKind = @enumFromInt(common_token_data.data);

                if (punctuator_kind == .OpenParen) {
                    const args = try parse_arguments(parser);

                    const lhs_clone = parser.allocator.create(expr.LeftHandSideExpression) catch return error.OutOfMemory;
                    lhs_clone.* = lhs.*;

                    const call_expr = parser.allocator.create(expr.CallExpression) catch return error.OutOfMemory;

                    if (lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW) {
                        const new_data = lhs.data.new;

                        if (new_data.tag == expr.NEW_EXPR_MEMBER) {
                            const member = new_data.data.member;

                            call_expr.* = expr.CallExpression{
                                .tag = expr.CALL_EXPR_COVER,
                                .data = .{
                                    .cover = .{
                                        .callee = member,
                                        .arguments = args,
                                    },
                                },
                            };
                        }
                    } else if (lhs.tag == expr.LEFT_HAND_SIDE_EXPR_CALL) {
                        const call_data = lhs.data.call;

                        call_expr.* = expr.CallExpression{
                            .tag = expr.CALL_EXPR_SIMPLE_CALL,
                            .data = .{
                                .simple_call = .{
                                    .callee = call_data,
                                    .arguments = args,
                                },
                            },
                        };
                    }

                    lhs.* = expr.LeftHandSideExpression{
                        .tag = expr.LEFT_HAND_SIDE_EXPR_CALL,
                        .data = .{
                            .call = call_expr,
                        },
                    };
                } else {
                    // TODO: handle things like abc()[x]
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    return lhs;
}

test "parse lhs" {
    const text = "obj.method(arg1, arg2)";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_lhs_expression(&parser)).*;

    std.debug.assert(result.tag == expr.LEFT_HAND_SIDE_EXPR_CALL);
    std.debug.assert(result.data.call.tag == expr.CALL_EXPR_COVER);
    std.debug.assert(result.data.call.data.cover.callee.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.call.data.cover.callee.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.call.data.cover.callee.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.call.data.cover.callee.data.property.object.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("obj")), 3),
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.call.data.cover.callee.data.property.property.name,
        testing.u8_array_to_string(@ptrCast(@constCast("method")), 6),
    ));

    std.debug.assert(result.data.call.data.cover.arguments.arguments.len == 2);
    std.debug.assert(result.data.call.data.cover.arguments.is_spread[0] == false);
    std.debug.assert(result.data.call.data.cover.arguments.is_spread[1] == false);

    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.call.data.cover.arguments.arguments.data[0].data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg1")), 4),
    ));

    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.call.data.cover.arguments.arguments.data[1].data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg2")), 4),
    ));

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}

test "parse call expr" {
    const text = "obj.method(arg1, arg2)";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_call_expression(&parser)).*;

    std.debug.assert(result.tag == expr.CALL_EXPR_COVER);
    std.debug.assert(result.data.cover.callee.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.cover.callee.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.callee.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.callee.data.property.object.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("obj")), 3),
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.callee.data.property.property.name,
        testing.u8_array_to_string(@ptrCast(@constCast("method")), 6),
    ));

    std.debug.assert(result.data.cover.arguments.arguments.len == 2);

    std.debug.assert(result.data.cover.arguments.is_spread[0] == false);
    std.debug.assert(result.data.cover.arguments.is_spread[1] == false);

    std.debug.assert(result.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[0].data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg1")), 4),
    ));

    std.debug.assert(result.data.cover.arguments.arguments.data[1].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[1].data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg2")), 4),
    ));

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}

test "parse call expr (spread)" {
    const text = "obj.method(arg1, ...args)";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_call_expression(&parser)).*;

    std.debug.assert(result.tag == expr.CALL_EXPR_COVER);
    std.debug.assert(result.data.cover.callee.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.cover.callee.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.callee.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.callee.data.property.object.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("obj")), 3),
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.callee.data.property.property.name,
        testing.u8_array_to_string(@ptrCast(@constCast("method")), 6),
    ));

    std.debug.assert(result.data.cover.arguments.arguments.len == 2);

    std.debug.assert(result.data.cover.arguments.is_spread[0] == false);
    std.debug.assert(result.data.cover.arguments.is_spread[1] == true);

    std.debug.assert(result.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[0].data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg1")), 4),
    ));

    std.debug.assert(result.data.cover.arguments.arguments.data[1].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[1].data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("args")), 4),
    ));

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}

test "parse arguments (empty)" {
    const text = "( )";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_arguments(&parser)).*;

    std.debug.assert(result.arguments.len == 0);
}

test "parse arguments (with args)" {
    const text = "(arg1, arg2)";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_arguments(&parser)).*;

    std.debug.assert(result.arguments.len == 2);

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}

test "parse arguments (with spread)" {
    const text = "(arg1, ...rest)";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_arguments(&parser)).*;

    std.debug.assert(result.arguments.len == 2);
    std.debug.assert(result.is_spread[0] == false);
    std.debug.assert(result.is_spread[1] == true);

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}

test "parse new expr" {
    const text = "new obj.prop[expr]";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_new_expression(&parser)).*;

    std.debug.assert(result.tag == expr.NEW_EXPR_NEW);
    std.debug.assert(result.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.new.data.member.tag == expr.MEMBER_EXPR_MEMBER);
    std.debug.assert(result.data.new.data.member.data.member.object.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.new.data.member.data.member.object.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.new.data.member.data.member.object.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
}

test "parse member expr" {
    const text = "obj.prop[expr]";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = (try parse_member_expression(&parser)).*;

    const obj_str = testing.u8_array_to_string(@ptrCast(@constCast("obj")), 3);
    const prop_str = testing.u8_array_to_string(@ptrCast(@constCast("prop")), 4);
    const expr_str = testing.u8_array_to_string(@ptrCast(@constCast("expr")), 4);

    std.debug.assert(result.tag == expr.MEMBER_EXPR_MEMBER);
    std.debug.assert(result.data.member.object.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.member.object.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.member.object.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.member.object.data.property.object.data.primary.data.identifier.data.identifier.name,
        obj_str,
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.member.object.data.property.property.name,
        prop_str,
    ));

    std.debug.assert(result.data.member.expr.data[0].tag == expr.ASSIGNMENT_EXPR_PRIMARY);
    std.debug.assert(result.data.member.expr.data[0].data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.member.expr.data[0].data.primary.data.identifier.data.identifier.name,
        expr_str,
    ));

    _text.free_string(str);
    _text.free_token_seq(tokens);
    deinit(&parser);
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

    std.debug.assert(testing.are_equal_strings(
        result.data.identifier.data.identifier.name,
        str,
    ));

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

    std.debug.assert(testing.are_equal_strings(
        result.data.literal.data.string.*,
        str,
    ));

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
