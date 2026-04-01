const std = @import("std");

const _text = @import("../text.zig");

const testing = @import("../../testing.zig");

const TokenSeq = _text.TokenSeq;
const Token = _text.Token;

const CommonTokenData = _text.CommonTokenData;
const IdentifierNameData = _text.IdentifierNameData;

const expr = @import("expressions.zig");

const p = @import("parser.zig");
const Parser = p.Parser;

const get_parse_result = @import("mod.zig").get_parse_result;

pub const MemberExpressionStarter = enum {
    Dot,
    OpenBracket,
};

fn is_assignment_operator(token: Token) ?expr.AssignmentOperator {
    if (token.kind != .CommonToken) {
        return null;
    }

    const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

    if (common_token_data.common_token_kind == .Punctuator) {
        const punctuator_kind: _text.PunctuatorKind = @enumFromInt(common_token_data.data);

        return switch (punctuator_kind) {
            .Assign => .Raw,
            .PlusAssign => .Plus,
            .MinusAssign => .Minus,
            .AsteriskAssign => .Star,
            .SlashAssign => .Slash,
            .PercentAssign => .Percent,
            .ExponentiationAssign => .Exponentiation,
            .LeftShiftAssign => .LeftShift,
            .RightShiftAssign => .RightShift,
            .UnsignedRightShiftAssign => .UnsignedRightShift,
            .BitwiseAndAssign => .BitwiseAnd,
            .BitwiseOrAssign => .BitwiseOr,
            .BitwiseXorAssign => .BitwiseXor,
            else => return null,
        };
    } else {
        return null;
    }
}

fn coerce_to_lhs(expression: *expr.AssignmentExpression, parser: *Parser) error{ OutOfMemory, UnexpectedToken }!*expr.LeftHandSideExpression {
    return switch (expression.tag) {
        expr.ASSIGNMENT_EXPR_PRIMARY => {
            const primary = expression.data.primary;

            switch (primary.tag) {
                expr.PRIMARY_EXPR_IDENTIFIER => {
                    const identifier_ref = primary.data.identifier;

                    const lhs = parser.allocator.create(expr.LeftHandSideExpression) catch return error.OutOfMemory;
                    const new = parser.allocator.create(expr.NewExpression) catch return error.OutOfMemory;
                    const member = parser.allocator.create(expr.MemberExpression) catch return error.OutOfMemory;
                    const new_primary = parser.allocator.create(expr.PrimaryExpression) catch return error.OutOfMemory;

                    primary.* = .{
                        .tag = expr.PRIMARY_EXPR_IDENTIFIER,
                        .data = .{
                            .identifier = identifier_ref,
                        },
                    };

                    member.* = .{
                        .tag = expr.MEMBER_EXPR_PRIMARY,
                        .data = .{
                            .primary = new_primary,
                        },
                    };

                    new.* = .{
                        .tag = expr.NEW_EXPR_MEMBER,
                        .data = .{
                            .member = member,
                        },
                    };

                    lhs.* = expr.LeftHandSideExpression{
                        .tag = expr.LEFT_HAND_SIDE_EXPR_NEW,
                        .data = .{
                            .new = new,
                        },
                    };

                    return lhs;
                },
                else => return error.UnexpectedToken,
            }
        },
        expr.ASSIGNMENT_EXPR_LHS => return expression.data.lhs,
        else => {
            std.debug.print("Cannot coerce expression to left-hand side expression: {any}\n", .{expression});
            return error.UnexpectedToken;
        },
    };
}

pub fn parse_expression(parser: *Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.Expression {
    var exprs = std.ArrayList(expr.AssignmentExpression).empty;

    const first_assignment_expr = try parse_assignment_expression(parser);

    try exprs.append(parser.allocator, first_assignment_expr.*);

    while (p.match(parser, Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.Comma),
        }),
    })) {
        p.skip_whitespace(parser);
        const assignment_expr = try parse_assignment_expression(parser);

        try exprs.append(parser.allocator, assignment_expr.*);
    }

    const expression = parser.allocator.create(expr.Expression) catch return error.OutOfMemory;

    const exprs_slice = exprs.toOwnedSlice(parser.allocator) catch return error.OutOfMemory;

    expression.* = .{
        .data = exprs_slice.ptr,
        .len = exprs_slice.len,
    };

    return expression;
}

test "parse simple expression" {
    const res = get_parse_result(expr.Expression, parse_expression, "a + b") catch |err| {
        std.debug.print("Error parsing expression: {any}\n", .{err});
        return;
    };

    std.debug.assert(res.len == 1);
    std.debug.assert(res.data[0].tag == expr.ASSIGNMENT_EXPR_BINARY);
    std.debug.assert(res.data[0].data.binary.operator == .Plus);

    // If we get this much, we're probs fine
}

pub fn parse_assignment_expression(parser: *Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.AssignmentExpression {
    const left = try parse_binary_expression(parser, 0);
    p.skip_whitespace(parser);

    const left_assignment = parser.allocator.create(expr.AssignmentExpression) catch return error.OutOfMemory;

    if (left.operator == .None) {
        if (left.left.data.unary.operator == .None) {
            left_assignment.* = expr.AssignmentExpression{
                .tag = expr.ASSIGNMENT_EXPR_LHS,
                .data = .{
                    .lhs = left.left.data.unary.operand.data.left_hand_side,
                },
            };
        } else {
            left_assignment.* = expr.AssignmentExpression{
                .tag = expr.ASSIGNMENT_EXPR_UNARY,
                .data = .{
                    .unary = left.left.data.unary,
                },
            };
        }
    } else {
        left_assignment.* = expr.AssignmentExpression{
            .tag = expr.ASSIGNMENT_EXPR_BINARY,
            .data = .{
                .binary = left,
            },
        };
    }

    const tok = p.peek(parser) orelse return left_assignment;

    if (is_assignment_operator(tok)) |op| {
        _ = p.next(parser);
        p.skip_whitespace(parser);

        const right = try parse_assignment_expression(parser);

        const lhs = try coerce_to_lhs(left_assignment, parser);

        const node = parser.allocator.create(expr.AssignmentExpression) catch return error.OutOfMemory;

        if (op == .Raw) {
            node.* = expr.AssignmentExpression{
                .tag = expr.ASSIGNMENT_EXPR_RAW,
                .data = .{
                    .raw_assignment = .{
                        .left = lhs,
                        .right = right,
                    },
                },
            };
        } else {
            node.* = expr.AssignmentExpression{
                .tag = expr.ASSIGNMENT_EXPR_OPERATOR,
                .data = .{
                    .operator_assignment = .{
                        .operator = op,
                        .left = lhs,
                        .right = right,
                    },
                },
            };
        }

        return node;
    }

    return left_assignment;

    // const prim = try parse_primary_expression(parser);

    // const assignment_expr = parser.allocator.create(expr.AssignmentExpression) catch return error.OutOfMemory;

    // assignment_expr.* = expr.AssignmentExpression{
    //     .tag = expr.ASSIGNMENT_EXPR_PRIMARY,
    //     .data = .{
    //         .primary = prim,
    //     },
    // };

    // return assignment_expr;
}

pub fn parse_primary_expression(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory, UnexpectedToken }!*expr.PrimaryExpression {
    const token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;

    const res = switch (token.kind) {
        .CommonToken => com: {
            const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

            break :com switch (common_token_data.common_token_kind) {
                .IdentifierName => idtfr: {
                    const identifier_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);

                    _ = p.next(parser);

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

                    _ = p.next(parser);

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

                    _ = p.next(parser);

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
                            _ = p.next(parser);

                            const inner = try parse_primary_expression(parser);

                            try p.expect_skip_whitespace(parser, Token{
                                .kind = .CommonToken,
                                .data = @intFromPtr(&CommonTokenData{
                                    .common_token_kind = .Punctuator,
                                    .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
                                }),
                            });

                            break :paren inner.*;
                        },
                        .OpenBracket => bracket: {
                            _ = p.next(parser);

                            var elems = std.ArrayList(expr.ArrayElement).empty;
                            defer elems.deinit(parser.allocator);

                            while (true) {
                                const next_token = p.peek(parser) orelse break;

                                if (next_token.kind == .Whitespace or next_token.kind == .LineTerminator) {
                                    _ = p.next(parser);
                                    continue;
                                }

                                if (next_token.kind != .CommonToken) {
                                    return error.UnexpectedToken;
                                }

                                const next_common_token_data: *CommonTokenData = @ptrFromInt(next_token.data);

                                if (next_common_token_data.common_token_kind == .Punctuator) {
                                    const next_punctuator_kind: _text.PunctuatorKind = @enumFromInt(next_common_token_data.data);

                                    if (next_punctuator_kind == .CloseBracket) {
                                        _ = p.next(parser);
                                        break;
                                    } else if (next_punctuator_kind == .Comma) {
                                        _ = p.next(parser);

                                        const ellision = parser.allocator.create(expr.ArrayElement) catch return error.OutOfMemory;

                                        ellision.* = expr.ArrayElement{
                                            .tag = expr.ARRAY_ELEMENT_ELLISION,
                                            .data = .{ .ellision = {} },
                                        };

                                        try elems.append(parser.allocator, ellision.*);
                                        continue;
                                    } else if (next_punctuator_kind == .Ellipsis) {
                                        _ = p.next(parser);

                                        const array_element = parser.allocator.create(expr.ArrayElement) catch return error.OutOfMemory;

                                        const rest_elem = try parse_assignment_expression(parser);

                                        array_element.* = expr.ArrayElement{
                                            .tag = expr.ARRAY_ELEMENT_SPREAD,
                                            .data = .{ .spread = rest_elem },
                                        };

                                        try elems.append(parser.allocator, array_element.*);

                                        try p.expect_skip_whitespace(parser, Token{
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

                                    p.expect_skip_whitespace(parser, Token{
                                        .kind = .CommonToken,
                                        .data = @intFromPtr(&CommonTokenData{
                                            .common_token_kind = .Punctuator,
                                            .data = @intFromEnum(_text.PunctuatorKind.Comma),
                                        }),
                                    }) catch try p.expect_skip_whitespace(parser, Token{
                                        .kind = .CommonToken,
                                        .data = @intFromPtr(&CommonTokenData{
                                            .common_token_kind = .Punctuator,
                                            .data = @intFromEnum(_text.PunctuatorKind.CloseBracket),
                                        }),
                                    });

                                    _ = p.next(parser);
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
                            std.debug.print("Unexpected punctuator in primary expression: {any}\n", .{punctuator_kind});
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

pub fn parse_member_expression(parser: *Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.MemberExpression {
    const prim = try parse_primary_expression(parser);

    const member_expr = parser.allocator.create(expr.MemberExpression) catch return error.OutOfMemory;
    member_expr.* = expr.MemberExpression{
        .tag = expr.MEMBER_EXPR_PRIMARY,
        .data = .{
            .primary = prim,
        },
    };

    while (p.peek(parser) != null) {
        const token = p.peek(parser).?;
        const member_expr_start = token_is_member_expr_start(token) orelse break;

        switch (member_expr_start) {
            .Dot => {
                _ = p.next(parser);

                const property_token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;

                if (property_token.kind != .CommonToken) {
                    return error.UnexpectedToken;
                }

                const property_common_token_data: *CommonTokenData = @ptrFromInt(property_token.data);

                if (property_common_token_data.common_token_kind != .IdentifierName) {
                    return error.UnexpectedToken;
                }

                const property_identifier_data: *IdentifierNameData = @ptrFromInt(property_common_token_data.data);

                _ = p.next(parser);

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
                _ = p.next(parser);

                const property_expr = try parse_assignment_expression(parser);

                const property_arr = parser.allocator.create([1]expr.AssignmentExpression) catch return error.OutOfMemory;
                property_arr.* = [_]expr.AssignmentExpression{property_expr.*};

                // const property_arr = [_]expr.AssignmentExpression{property_expr.*};

                const property_expr_wrapper = parser.allocator.create(expr.Expression) catch return error.OutOfMemory;
                property_expr_wrapper.* = expr.Expression{
                    .data = &property_arr.*,
                    .len = 1,
                };

                try p.expect_skip_whitespace(parser, Token{
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

pub fn parse_new_expression(parser: *Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.NewExpression {
    if (p.peek(parser) == null) {
        return error.UnexpectedEndOfTokens;
    }

    const token = p.peek(parser).?;

    if (token.kind == .CommonToken) {
        const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

        if (common_token_data.common_token_kind == .IdentifierName) {
            const identifier_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);

            if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("new")), 3))) {
                _ = p.next(parser);

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

test "parse new expr (member kind)" {
    const res = get_parse_result(expr.NewExpression, parse_new_expression, "a.b") catch |err| {
        std.debug.print("Error parsing new expression: {any}\n", .{err});
        return;
    };

    std.debug.assert(res.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(res.data.member.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(res.data.member.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(res.data.member.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);
    std.debug.assert(res.data.member.data.property.object.data.primary.data.identifier.tag == expr.IDENTIFIER_REF_IDENTIFIER);
    std.debug.assert(
        testing.are_equal_strings(
            res.data.member.data.property.object.data.primary.data.identifier.data.identifier.name,
            testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
        ),
    );
    std.debug.assert(res.data.member.data.property.property.name.len == 1);
    std.debug.assert(
        testing.are_equal_strings(
            res.data.member.data.property.property.name,
            testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
        ),
    );
}

test "parse expr (console.log)" {
    const res = get_parse_result(expr.Expression, parse_expression, "console.log") catch |err| {
        std.debug.print("Error parsing expression: {any}\n", .{err});
        return;
    };

    std.debug.assert(res.len == 1);
    std.debug.assert(res.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(res.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(res.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(res.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(res.data[0].data.lhs.data.new.data.member.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(res.data[0].data.lhs.data.new.data.member.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);
    std.debug.assert(
        testing.are_equal_strings(
            res.data[0].data.lhs.data.new.data.member.data.property.object.data.primary.data.identifier.data.identifier.name,
            testing.u8_array_to_string(@ptrCast(@constCast("console")), 7),
        ),
    );
    std.debug.assert(res.data[0].data.lhs.data.new.data.member.data.property.property.name.len == 3);
    std.debug.assert(
        testing.are_equal_strings(
            res.data[0].data.lhs.data.new.data.member.data.property.property.name,
            testing.u8_array_to_string(@ptrCast(@constCast("log")), 3),
        ),
    );
}

pub fn parse_arguments(parser: *Parser) !*expr.Arguments {
    if (!p.match(parser, Token{
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

    while (!p.match(parser, Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
        }),
    })) {
        const next_token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;

        if (next_token.kind == .Whitespace or next_token.kind == .LineTerminator) {
            _ = p.next(parser);
            continue;
        }

        if (next_token.kind == .CommonToken) {
            const next_common_token_data: *CommonTokenData = @ptrFromInt(next_token.data);

            if (next_common_token_data.common_token_kind == .Punctuator) {
                const next_punctuator_kind: _text.PunctuatorKind = @enumFromInt(next_common_token_data.data);

                if (next_punctuator_kind == .Comma) {
                    _ = p.next(parser);
                    continue;
                }
            }

            if (next_common_token_data.common_token_kind == .Punctuator) {
                const next_punctuator_kind: _text.PunctuatorKind = @enumFromInt(next_common_token_data.data);

                if (next_punctuator_kind == .Ellipsis) {
                    _ = p.next(parser);
                    try is_spread.append(parser.allocator, true);
                } else {
                    try is_spread.append(parser.allocator, false);
                }
            } else {
                try is_spread.append(parser.allocator, false);
            }

            p.skip_whitespace(parser);

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
    p.skip_whitespace(parser);

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

pub fn parse_lhs_expression(parser: *Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.LeftHandSideExpression {
    const new_expr = try parse_new_expression(parser);

    const lhs = parser.allocator.create(expr.LeftHandSideExpression) catch return error.OutOfMemory;
    lhs.* = expr.LeftHandSideExpression{
        .tag = expr.LEFT_HAND_SIDE_EXPR_NEW,
        .data = .{
            .new = new_expr,
        },
    };

    p.skip_whitespace(parser);

    while (p.peek(parser) != null) {
        const token = p.peek(parser).?;

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

pub fn is_unary_operator(token: Token, is_prefix: bool) ?expr.UnaryOperator {
    if (token.kind != .CommonToken) {
        return null;
    }

    const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

    if (common_token_data.common_token_kind == .Punctuator) {
        const punctuator_kind: _text.PunctuatorKind = @enumFromInt(common_token_data.data);

        return switch (punctuator_kind) {
            .Increment => if (is_prefix) .PrefixIncrement else .PostfixIncrement,
            .Decrement => if (is_prefix) .PrefixDecrement else .PostfixDecrement,
            .Plus => if (is_prefix) .Plus else return null,
            .Minus => if (is_prefix) .Minus else return null,
            .Not => if (is_prefix) .LogicalNot else return null,
            .BitwiseNot => if (is_prefix) .BitwiseNot else return null,
            else => return null,
        };
    } else if (common_token_data.common_token_kind == .IdentifierName) {
        const identifier_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);

        if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("typeof")), 6))) {
            return if (is_prefix) .TypeOf else null;
        } else if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("void")), 4))) {
            return if (is_prefix) .Void else null;
        } else if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("delete")), 6))) {
            return if (is_prefix) .Delete else null;
        } else if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("await")), 5))) {
            return if (is_prefix) .Await else null;
        } else {
            return null;
        }
    } else {
        return null;
    }
}

pub fn parse_unary_expression(parser: *Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.UnaryExpression {
    const next_token = p.peek(parser).?;

    if (is_unary_operator(next_token, true)) |unary_op| {
        _ = p.next(parser);
        p.skip_whitespace(parser);

        const operand = try parse_unary_expression(parser);

        const unary_expr_or_lhs = parser.allocator.create(expr.UnaryExpressionOrLHS) catch return error.OutOfMemory;
        unary_expr_or_lhs.* = .{
            .tag = expr.UNARY_EXPR_OR_LHS_UNARY,
            .data = .{
                .unary = operand,
            },
        };

        const unary_expr = parser.allocator.create(expr.UnaryExpression) catch return error.OutOfMemory;
        unary_expr.* = expr.UnaryExpression{
            .operator = unary_op,
            .operand = unary_expr_or_lhs,
        };

        return unary_expr;
    }

    const operand = try parse_lhs_expression(parser);

    const unary_expr_or_lhs = parser.allocator.create(expr.UnaryExpressionOrLHS) catch return error.OutOfMemory;
    const unary_expr = parser.allocator.create(expr.UnaryExpression) catch return error.OutOfMemory;

    if (p.peek(parser)) |token| {
        if (is_unary_operator(token, false)) |operator| {
            _ = p.next(parser);

            unary_expr_or_lhs.* = .{
                .tag = expr.UNARY_EXPR_OR_LHS_LHS,
                .data = .{
                    .left_hand_side = operand,
                },
            };

            unary_expr.* = expr.UnaryExpression{
                .operator = operator,
                .operand = unary_expr_or_lhs,
            };

            return unary_expr;
        }
    }

    unary_expr_or_lhs.* = .{
        .tag = expr.UNARY_EXPR_OR_LHS_LHS,
        .data = .{
            .left_hand_side = operand,
        },
    };

    unary_expr.* = expr.UnaryExpression{
        .operator = .None,
        .operand = unary_expr_or_lhs,
    };

    return unary_expr;
}

pub fn is_binary_operator(token: Token) ?expr.BinaryOperator {
    if (token.kind != .CommonToken) {
        return null;
    }

    const common_token_data: *CommonTokenData = @ptrFromInt(token.data);

    if (common_token_data.common_token_kind == .Punctuator) {
        const punctuator_kind: _text.PunctuatorKind = @enumFromInt(common_token_data.data);

        return switch (punctuator_kind) {
            .Plus => .Plus,
            .Minus => .Minus,
            .Asterisk => .Star,
            .Slash => .Slash,
            .Percent => .Percent,
            .Exponentiation => .Exponentiation,
            .BitwiseAnd => .BitwiseAnd,
            .BitwiseOr => .BitwiseOr,
            .BitwiseXor => .BitwiseXor,
            .LogicalAnd => .LogicalAnd,
            .LogicalOr => .LogicalOr,
            .Equals => .Equal,
            .NotEquals => .NotEqual,
            .StrictEquals => .StrictEqual,
            .StrictNotEquals => .StrictNotEqual,
            .GreaterThan => .GreaterThan,
            .GreaterThanEqual => .GreaterThanOrEqual,
            .LessThan => .LessThan,
            .LessThanEqual => .LessThanOrEqual,
            else => return null,
        };
    } else if (common_token_data.common_token_kind == .IdentifierName) {
        const identifier_data: *IdentifierNameData = @ptrFromInt(common_token_data.data);

        if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("instanceof")), 10))) {
            return .InstanceOf;
        } else if (testing.are_equal_strings(identifier_data.name, testing.u8_array_to_string(@ptrCast(@constCast("in")), 2))) {
            return .In;
        } else {
            return null;
        }
    } else {
        return null;
    }
}

pub fn precedence_of_binop(operator: expr.BinaryOperator) u8 {
    return switch (operator) {
        .Star, .Slash, .Percent => 14,
        .Exponentiation => 15,
        .Plus, .Minus => 13,
        .LeftShift, .RightShift, .UnsignedRightShift => 12,
        .LessThan, .LessThanOrEqual, .GreaterThan, .GreaterThanOrEqual, .In, .InstanceOf => 11,
        .Equal, .NotEqual, .StrictEqual, .StrictNotEqual => 10,
        .BitwiseAnd => 9,
        .BitwiseXor => 8,
        .BitwiseOr => 7,
        .LogicalAnd => 6,
        .LogicalOr => 5,
        else => 0,
    };
}

pub fn parse_binary_expression(parser: *Parser, min_precedence: u8) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*expr.BinaryExpression {
    const left = try parse_unary_expression(parser);
    p.skip_whitespace(parser);

    const binary_or_unary = parser.allocator.create(expr.BinaryOrUnaryExpression) catch return error.OutOfMemory;
    binary_or_unary.* = .{
        .tag = expr.BINARY_OR_UNARY_UNARY,
        .data = .{
            .unary = left,
        },
    };

    const unary_or_null = parser.allocator.create(expr.UnaryExpressionOrNull) catch return error.OutOfMemory;
    unary_or_null.* = .{
        .tag = expr.UNARY_EXPR_OR_NULL_UNARY,
        .data = .{
            .none = {},
        },
    };

    const binary = parser.allocator.create(expr.BinaryExpression) catch return error.OutOfMemory;

    const binary_or_unary_clone = parser.allocator.create(expr.BinaryOrUnaryExpression) catch return error.OutOfMemory;
    binary_or_unary_clone.* = binary_or_unary.*;

    binary.* = expr.BinaryExpression{
        .operator = expr.BinaryOperator.None,
        .left = binary_or_unary_clone,
        .right = unary_or_null,
    };

    while (p.peek(parser)) |token| {
        const operator = is_binary_operator(token) orelse break;

        const precedence = precedence_of_binop(operator);

        if (precedence < min_precedence) {
            break;
        }

        // consume operator
        _ = p.next(parser);

        const right = try parse_unary_expression(parser);
        p.skip_whitespace(parser);

        const right_or_null = parser.allocator.create(expr.UnaryExpressionOrNull) catch return error.OutOfMemory;
        right_or_null.* = .{
            .tag = expr.UNARY_EXPR_OR_NULL_UNARY,
            .data = .{
                .unary = right,
            },
        };

        const new_left = parser.allocator.create(expr.BinaryOrUnaryExpression) catch return error.OutOfMemory;
        new_left.* = binary_or_unary.*;

        binary.* = expr.BinaryExpression{
            .operator = operator,
            .left = new_left,
            .right = right_or_null,
        };

        const binary_clone = parser.allocator.create(expr.BinaryExpression) catch return error.OutOfMemory;
        binary_clone.* = binary.*;

        binary_or_unary.* = .{
            .tag = expr.BINARY_OR_UNARY_BINARY,
            .data = .{
                .binary = binary_clone,
            },
        };
    }

    return binary;
}

test "parse assignment (raw)" {
    const result = try get_parse_result(
        expr.AssignmentExpression,
        parse_assignment_expression,
        "abc = 1 + 2",
    );

    std.debug.assert(result.tag == expr.ASSIGNMENT_EXPR_RAW);
    std.debug.assert(result.data.raw_assignment.left.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.raw_assignment.left.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.raw_assignment.left.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.raw_assignment.left.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.raw_assignment.left.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("abc")), 3),
    ));

    std.debug.assert(result.data.raw_assignment.right.tag == expr.ASSIGNMENT_EXPR_BINARY);
    std.debug.assert(result.data.raw_assignment.right.data.binary.operator == .Plus);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.tag == expr.BINARY_OR_UNARY_UNARY);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.raw_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 1);

    std.debug.assert(result.data.raw_assignment.right.data.binary.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.raw_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 2);
}

test "parse assignment (operator)" {
    const result = try get_parse_result(
        expr.AssignmentExpression,
        parse_assignment_expression,
        "abc += 1 + 2",
    );

    std.debug.assert(result.tag == expr.ASSIGNMENT_EXPR_OPERATOR);

    std.debug.assert(result.data.operator_assignment.operator == .Plus);
    std.debug.assert(result.data.operator_assignment.left.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.operator_assignment.left.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.operator_assignment.left.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.operator_assignment.left.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.operator_assignment.left.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("abc")), 3),
    ));

    std.debug.assert(result.data.operator_assignment.right.tag == expr.ASSIGNMENT_EXPR_BINARY);
    std.debug.assert(result.data.operator_assignment.right.data.binary.operator == .Plus);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.tag == expr.BINARY_OR_UNARY_UNARY);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.operator_assignment.right.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 1);

    std.debug.assert(result.data.operator_assignment.right.data.binary.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.operator_assignment.right.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 2);
}

test "parse binary (basic)" {
    const text = "1 + 2";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = try parse_binary_expression(&parser, 0);

    std.debug.assert(result.operator == .Plus);
    std.debug.assert(result.left.tag == expr.BINARY_OR_UNARY_UNARY);
    std.debug.assert(result.left.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.left.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.left.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.left.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 1);

    std.debug.assert(result.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 2);
}

test "parse binary (medium)" {
    const text = "1 + 2 * 3";
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = try parse_binary_expression(&parser, 0);

    std.debug.assert(result.operator == .Star);

    std.debug.assert(result.left.tag == expr.BINARY_OR_UNARY_BINARY);
    std.debug.assert(result.left.data.binary.operator == .Plus);
    std.debug.assert(result.left.data.binary.left.tag == expr.BINARY_OR_UNARY_UNARY);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.left.data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 1);

    std.debug.assert(result.left.data.binary.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.left.data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 2);

    std.debug.assert(result.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 3);
}

test "parse unary" {
    const result = try get_parse_result(
        expr.UnaryExpression,
        parse_unary_expression,
        "++abc",
    );

    std.debug.assert(result.operator == .PrefixIncrement);
    std.debug.assert(result.operand.tag == expr.UNARY_EXPR_OR_LHS_UNARY);
    std.debug.assert(result.operand.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.operand.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.operand.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.operand.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.operand.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);
    std.debug.assert(testing.are_equal_strings(
        result.operand.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("abc")), 3),
    ));
}

test "parse lhs" {
    const result = try get_parse_result(
        expr.LeftHandSideExpression,
        parse_lhs_expression,
        "obj.method(arg1, arg2)",
    );

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

    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg1")), 4),
    ));

    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.call.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.call.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg2")), 4),
    ));
}

test "parse call expr" {
    const result = try get_parse_result(
        expr.CallExpression,
        parse_call_expression,
        "obj.method(arg1, arg2)",
    );

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

    std.debug.assert(result.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg1")), 4),
    ));

    std.debug.assert(result.data.cover.arguments.arguments.data[1].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg2")), 4),
    ));
}

test "parse call expr (spread)" {
    const result = try get_parse_result(
        expr.CallExpression,
        parse_call_expression,
        "obj.method(arg1, ...args)",
    );

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

    std.debug.assert(result.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("arg1")), 4),
    ));

    std.debug.assert(result.data.cover.arguments.arguments.data[1].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.cover.arguments.arguments.data[1].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("args")), 4),
    ));
}

test "parse arguments (empty)" {
    const result = try get_parse_result(
        expr.Arguments,
        parse_arguments,
        "()",
    );

    std.debug.assert(result.arguments.len == 0);
}

test "parse arguments (with args)" {
    const result = try get_parse_result(
        expr.Arguments,
        parse_arguments,
        "(arg1, arg2)",
    );

    std.debug.assert(result.arguments.len == 2);
}

test "parse arguments (with spread)" {
    const result = try get_parse_result(
        expr.Arguments,
        parse_arguments,
        "(arg1, ...rest)",
    );

    std.debug.assert(result.arguments.len == 2);
    std.debug.assert(result.is_spread[0] == false);
    std.debug.assert(result.is_spread[1] == true);
}

test "parse new expr" {
    const result = try get_parse_result(
        expr.NewExpression,
        parse_new_expression,
        "new obj.prop[expr]",
    );

    std.debug.assert(result.tag == expr.NEW_EXPR_NEW);
    std.debug.assert(result.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.new.data.member.tag == expr.MEMBER_EXPR_MEMBER);
    std.debug.assert(result.data.new.data.member.data.member.object.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.new.data.member.data.member.object.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.new.data.member.data.member.object.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);
}

test "parse member expr" {
    const result = try get_parse_result(
        expr.MemberExpression,
        parse_member_expression,
        "obj.prop[expr]",
    );

    std.debug.assert(result.tag == expr.MEMBER_EXPR_MEMBER);
    std.debug.assert(result.data.member.object.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.member.object.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.member.object.data.property.object.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.member.object.data.property.object.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("obj")), 3),
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.member.object.data.property.property.name,
        testing.u8_array_to_string(@ptrCast(@constCast("prop")), 4),
    ));

    std.debug.assert(result.data.member.expr.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.member.expr.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.member.expr.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.member.expr.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.member.expr.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_IDENTIFIER);

    std.debug.assert(testing.are_equal_strings(
        result.data.member.expr.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("expr")), 4),
    ));
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
    p.deinit(&parser);
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

    var parser = p.init(TokenSeq{
        .data = &tokens,
        .len = 1,
    });

    const result = (try parse_primary_expression(&parser)).*;

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.literal.data.number.value == 42.0);

    p.deinit(&parser);
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

    var parser = p.init(TokenSeq{
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
    p.deinit(&parser);
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
    std.debug.assert(result.data.array.elements.data[0].data.expression.tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 1.0);

    std.debug.assert(result.data.array.elements.data[1].tag == expr.ARRAY_ELEMENT_EXPR);
    std.debug.assert(result.data.array.elements.data[1].data.expression.tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 2.0);

    p.deinit(&parser);
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
    std.debug.assert(result.data.array.elements.data[0].data.expression.tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[0].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 1.0);

    std.debug.assert(result.data.array.elements.data[1].tag == expr.ARRAY_ELEMENT_EXPR);
    std.debug.assert(result.data.array.elements.data[1].data.expression.tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.array.elements.data[1].data.expression.data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 2.0);

    _text.free_string(str);
    _text.free_token_seq(tokens);
    p.deinit(&parser);
}
