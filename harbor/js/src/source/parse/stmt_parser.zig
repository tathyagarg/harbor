const std = @import("std");

const _text = @import("../text.zig");
const CommonTokenData = _text.CommonTokenData;
const IdentifierNameData = _text.IdentifierNameData;

const p = @import("parser.zig");
const Parser = p.Parser;

const stmt = @import("statements.zig");
const expr = @import("expressions.zig");
const exp_parser = @import("exp_parser.zig");

const testing = @import("../../testing.zig");

const get_parse_result = @import("mod.zig").get_parse_result;

fn is_keyword(data: *CommonTokenData, keyword: []const u8) bool {
    if (data.common_token_kind != .IdentifierName) {
        return false;
    }

    const identifier_data: *IdentifierNameData = @ptrFromInt(data.data);
    return identifier_data.name.len == keyword.len and
        testing.are_equal_strings_pure(identifier_data.name, keyword);
}

pub fn parse_script(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.Script {
    var body = std.ArrayList(stmt.StatementOrDeclaration).empty;
    defer body.deinit(parser.allocator);

    while (p.peek(parser) != null) {
        const statement_or_declaration = try parse_statement_or_declaration(parser);
        try body.append(parser.allocator, statement_or_declaration.*);
    }

    const script = try parser.allocator.create(stmt.Script);
    const slice = try body.toOwnedSlice(parser.allocator);

    const body_data = try parser.allocator.create(_text.Seq(stmt.StatementOrDeclaration));
    body_data.* = _text.Seq(stmt.StatementOrDeclaration){
        .data = slice.ptr,
        .len = slice.len,
    };

    script.* = stmt.Script{
        .body = body_data.*,
    };

    return script;
}

test "parse basic script" {
    const result = try get_parse_result(stmt.Script, parse_script, "let a;");

    std.debug.assert(result.body.len == 1);
    std.debug.assert(result.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));
    std.debug.assert(result.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

pub fn parse_statement(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.Statement {
    p.skip_whitespace(parser);

    const token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;

    if (token.kind != .CommonToken) {
        const statement = try parser.allocator.create(stmt.Statement);
        statement.* = stmt.Statement{
            .tag = stmt.STATEMENT_EXPR_STATEMENT,
            .data = .{
                .expr_statement = try parse_expression_statement(parser),
            },
        };

        return statement;
    }

    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (data.common_token_kind == .IdentifierName) {
        const statement = try parser.allocator.create(stmt.Statement);
        if (is_keyword(data, "if")) {
            const if_stmt = try parse_if_statement(parser);

            statement.* = stmt.Statement{
                .tag = stmt.STATEMENT_IF_STATEMENT,
                .data = .{
                    .if_statement = if_stmt,
                },
            };

            return statement;
        }

        if (is_keyword(data, "while")) {
            const while_stmt = try parse_while_statement(parser);
            // const breakable_statement = try parser.allocator.create(stmt.BreakableStatement);

            // breakable_statement.* = stmt.BreakableStatement{
            //     .tag = stmt.BREAKABLE_STATEMENT_WHILE,
            //     .data = .{
            //         .while_ = while_stmt,
            //     },
            // };

            statement.* = stmt.Statement{
                .tag = stmt.STATEMENT_WHILE,
                .data = .{
                    .while_ = while_stmt,
                },
            };

            return statement;
        }

        if (is_keyword(data, "do")) {
            const do_while_stmt = try parse_do_while_statement(parser);
            // const breakable_statement = try parser.allocator.create(stmt.BreakableStatement);

            // breakable_statement.* = stmt.BreakableStatement{
            //     .tag = stmt.BREAKABLE_STATEMENT_DO_WHILE,
            //     .data = .{
            //         .do_while = do_while_stmt,
            //     },
            // };

            statement.* = stmt.Statement{
                .tag = stmt.STATEMENT_DO_WHILE,
                .data = .{
                    .do_while = do_while_stmt,
                },
            };

            return statement;
        }

        // if (is_keyword(name, "for")) return parse_for_statement(parser);
        if (is_keyword(data, "return")) {
            _ = p.next(parser);
            const value_data = try parser.allocator.create(stmt.MaybeExpression);
            value_data.* = stmt.MaybeExpression{
                .has_value = false,
                .value = .{
                    .none = {},
                },
            };

            if (!p.match(parser, _text.Token{
                .kind = .CommonToken,
                .data = @intFromPtr(&CommonTokenData{
                    .common_token_kind = .Punctuator,
                    .data = @intFromEnum(_text.PunctuatorKind.Semicolon),
                }),
            })) {
                const ret_expr = exp_parser.parse_expression(parser) catch return error.UnexpectedEndOfTokens;

                value_data.* = stmt.MaybeExpression{
                    .has_value = true,
                    .value = .{
                        .value = ret_expr.*,
                    },
                };

                if (!p.match(parser, _text.Token{
                    .kind = .CommonToken,
                    .data = @intFromPtr(&CommonTokenData{
                        .common_token_kind = .Punctuator,
                        .data = @intFromEnum(_text.PunctuatorKind.Semicolon),
                    }),
                })) {
                    return error.UnexpectedEndOfTokens;
                }

                _ = p.next(parser);
            }

            statement.* = stmt.Statement{
                .tag = stmt.STATEMENT_RETURN_STATEMENT,
                .data = .{
                    .return_statement = value_data,
                },
            };

            return statement;
        }
        // if (is_keyword(name, "break")) return parse_break_statement(parser);
        // if (is_keyword(name, "continue")) return parse_continue_statement(parser);
        // if (is_keyword(name, "var")) return parse_var_statement(parser);
        // if (is_keyword(name, "try")) return parse_try_statement(parser);
        // if (is_keyword(name, "with")) return parse_with_statement(parser);

        // if (is_keyword(name, "function")) return parse_function_declaration(parser);
    }

    if (data.common_token_kind == .Punctuator) {
        const punct: _text.PunctuatorKind = @enumFromInt(data.data);

        if (punct == .OpenBrace) {
            const statement = try parser.allocator.create(stmt.Statement);
            statement.* = stmt.Statement{
                .tag = stmt.STATEMENT_BLOCK_STATEMENT,
                .data = .{
                    .block_statement = try parse_block_statement(parser),
                },
            };

            return statement;
        }
        if (punct == .Semicolon) {
            p.expect_skip_whitespace(parser, _text.Token{
                .kind = .CommonToken,
                .data = @intFromPtr(&CommonTokenData{
                    .common_token_kind = .Punctuator,
                    .data = @intFromEnum(_text.PunctuatorKind.Semicolon),
                }),
            }) catch return error.UnexpectedEndOfTokens;
            _ = p.next(parser);

            const statement = try parser.allocator.create(stmt.Statement);
            statement.* = stmt.Statement{
                .tag = stmt.STATEMENT_EMPTY_STATEMENT,
                .data = .{
                    .empty_statement = {},
                },
            };

            return statement;
        }
    }

    const statement = try parser.allocator.create(stmt.Statement);
    statement.* = stmt.Statement{
        .tag = stmt.STATEMENT_EXPR_STATEMENT,
        .data = .{
            .expr_statement = try parse_expression_statement(parser),
        },
    };

    return statement;
}

test "parse return statement (with value)" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "return 42;");

    std.debug.assert(result.tag == stmt.STATEMENT_RETURN_STATEMENT);
    std.debug.assert(result.data.return_statement.has_value == true);
    std.debug.assert(result.data.return_statement.value.value.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.return_statement.value.value.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.return_statement.value.value.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.return_statement.value.value.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.return_statement.value.value.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.return_statement.value.value.data[0].data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.return_statement.value.value.data[0].data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 42);
}

pub fn parse_expression_statement(parser: *Parser) error{UnexpectedEndOfTokens}!*expr.Expression {
    const expression = exp_parser.parse_expression(parser) catch return error.UnexpectedEndOfTokens;

    p.skip_whitespace(parser);

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.Semicolon),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    return expression;
}

test "parse expr statement" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "a + b;");

    std.debug.assert(result.tag == stmt.STATEMENT_EXPR_STATEMENT);
    std.debug.assert(result.data.expr_statement.len == 1);
    std.debug.assert(result.data.expr_statement.data[0].tag == expr.ASSIGNMENT_EXPR_BINARY);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.left.tag == expr.BINARY_OR_UNARY_UNARY);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.left.data.unary.operator == .None);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.left.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.left.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.left.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.expr_statement.data[0].data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));

    std.debug.assert(result.data.expr_statement.data[0].data.binary.operator == .Plus);

    std.debug.assert(result.data.expr_statement.data[0].data.binary.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.right.data.unary.operator == .None);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.expr_statement.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.expr_statement.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
}

pub fn parse_block_statement(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.BlockStatement {
    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.OpenBrace),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    var items = std.ArrayList(stmt.StatementOrDeclaration).empty;
    defer items.deinit(parser.allocator);

    while (!p.match(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.CloseBrace),
        }),
    })) {
        const statement = try parse_statement_or_declaration(parser);
        try items.append(parser.allocator, statement.*);

        p.skip_whitespace(parser);
    }

    const block = try parser.allocator.create(stmt.BlockStatement);
    const slice = try items.toOwnedSlice(parser.allocator);

    block.* = stmt.BlockStatement{
        .body = .{
            .data = slice.ptr,
            .len = slice.len,
        },
    };

    return block;
}

test "parse block statement" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "{ let a; }");

    std.debug.assert(result.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.block_statement.body.len == 1);
    std.debug.assert(result.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

test "parse block statement (empty)" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "{ }");

    std.debug.assert(result.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.block_statement.body.len == 0);
}

test "parse block statement (multiple statements)" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "{ let a; let b; }");

    std.debug.assert(result.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.block_statement.body.len == 2);

    std.debug.assert(result.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));
    std.debug.assert(result.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);

    std.debug.assert(result.data.block_statement.body.data[1].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.block_statement.body.data[1].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.block_statement.body.data[1].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.block_statement.body.data[1].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.block_statement.body.data[1].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
    std.debug.assert(result.data.block_statement.body.data[1].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

fn is_declaration_start(token: _text.Token) bool {
    if (token.kind != .CommonToken) {
        return false;
    }

    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (data.common_token_kind != .IdentifierName) {
        return false;
    }

    return is_keyword(data, "function") or
        is_keyword(data, "let") or
        is_keyword(data, "const");
}

pub fn parse_statement_or_declaration(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.StatementOrDeclaration {
    const statement_or_declaration = try parser.allocator.create(stmt.StatementOrDeclaration);

    if (is_declaration_start(p.peek(parser) orelse return error.UnexpectedEndOfTokens)) {
        const declaration = try parse_declaration(parser);

        statement_or_declaration.* = stmt.StatementOrDeclaration{
            .tag = stmt.STATEMENT_OR_DECLARATION_DECLARATION,
            .data = .{
                .declaration = declaration,
            },
        };
    } else {
        const statement = try parse_statement(parser);

        statement_or_declaration.* = stmt.StatementOrDeclaration{
            .tag = stmt.STATEMENT_OR_DECLARATION_STATEMENT,
            .data = .{
                .statement = statement,
            },
        };
    }

    return statement_or_declaration;
}

pub fn parse_declaration(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.Declaration {
    const token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;

    if (token.kind != .CommonToken) {
        return error.UnexpectedEndOfTokens;
    }

    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (data.common_token_kind == .IdentifierName) {
        if (is_keyword(data, "function")) {
            const function = try parse_function_declaration(parser);

            const declaration = try parser.allocator.create(stmt.Declaration);
            declaration.* = stmt.Declaration{
                .tag = stmt.DECLARATION_FUNCTION_DECLARATION,
                .data = .{
                    .function_declaration = function,
                },
            };

            return declaration;
        }
        if (is_keyword(data, "let") or is_keyword(data, "const")) {
            const lexical_declaration = try parse_lexical_declaration(parser);

            const declaration = try parser.allocator.create(stmt.Declaration);
            declaration.* = stmt.Declaration{
                .tag = stmt.DECLARATION_LEXICAL_DECLARATION,
                .data = .{
                    .lexical_declaration = lexical_declaration,
                },
            };

            return declaration;
        }
    }

    return error.UnexpectedEndOfTokens;
}

pub fn parse_function_declaration(parser: *Parser) error{UnexpectedEndOfTokens}!*stmt.HoistableDeclaration {
    _ = .{parser};
    return error.UnexpectedEndOfTokens;
}

pub fn parse_lexical_declaration(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.LexicalDeclaration {
    const token = p.next(parser) orelse return error.UnexpectedEndOfTokens;

    if (token.kind != .CommonToken) return error.UnexpectedEndOfTokens;
    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (data.common_token_kind != .IdentifierName) return error.UnexpectedEndOfTokens;
    const is_const = is_keyword(data, "const");

    var declarations = std.ArrayList(stmt.LexicalBinding).empty;
    defer declarations.deinit(parser.allocator);

    while (true) {
        const binding = try parse_lexical_binding(parser);
        try declarations.append(parser.allocator, binding.*);

        if (!p.match(parser, _text.Token{
            .kind = .CommonToken,
            .data = @intFromPtr(&CommonTokenData{
                .common_token_kind = .Punctuator,
                .data = @intFromEnum(_text.PunctuatorKind.Comma),
            }),
        })) {
            break;
        }
    }

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.Semicolon),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const lexical_declaration = try parser.allocator.create(stmt.LexicalDeclaration);
    const slice = try declarations.toOwnedSlice(parser.allocator);

    lexical_declaration.* = stmt.LexicalDeclaration{
        .is_const = is_const,
        .declarations = .{
            .data = slice.ptr,
            .len = slice.len,
        },
    };

    return lexical_declaration;
}

test "parse lexical declaration (basic)" {
    const result = try get_parse_result(stmt.Declaration, parse_declaration, "let a;");

    std.debug.assert(result.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));
}

test "parse lexical declaration (with initializer)" {
    const result = try get_parse_result(stmt.Declaration, parse_declaration, "const a = 42;");

    std.debug.assert(result.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.lexical_declaration.is_const == true);
    std.debug.assert(result.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.has_value == true);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.value.value.data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 42);
}

test "parse lexical declaration (multiple)" {
    const result = try get_parse_result(stmt.Declaration, parse_declaration, "let a, b = 42;");

    std.debug.assert(result.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.lexical_declaration.declarations.len == 2);

    std.debug.assert(testing.are_equal_strings(
        result.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));
    std.debug.assert(result.data.lexical_declaration.declarations.data[0].initializer.has_value == false);

    std.debug.assert(testing.are_equal_strings(
        result.data.lexical_declaration.declarations.data[1].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.has_value == true);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.lexical_declaration.declarations.data[1].initializer.value.value.data.lhs.data.new.data.member.data.primary.data.literal.data.number.value == 42);
}

test "parse lexical declaration (missing semicolon)" {
    const result = get_parse_result(stmt.Declaration, parse_declaration, "let a");

    std.debug.assert(result == error.UnexpectedEndOfTokens);
}

pub fn parse_lexical_binding(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.LexicalBinding {
    const token = p.next(parser) orelse return error.UnexpectedEndOfTokens;

    if (token.kind != .CommonToken) return error.UnexpectedEndOfTokens;
    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (data.common_token_kind != .IdentifierName) return error.UnexpectedEndOfTokens;
    const name: *IdentifierNameData = @ptrFromInt(data.data);

    const initializer: *stmt.MaybeAssignmentExpression = try parser.allocator.create(stmt.MaybeAssignmentExpression);
    initializer.* = stmt.MaybeAssignmentExpression{
        .has_value = false,
        .value = .{
            .none = {},
        },
    };

    if (p.match(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.Assign),
        }),
    })) {
        p.skip_whitespace(parser);

        const init_expr = exp_parser.parse_assignment_expression(parser) catch return error.UnexpectedEndOfTokens;

        initializer.* = stmt.MaybeAssignmentExpression{
            .has_value = true,
            .value = .{
                .value = init_expr.*,
            },
        };
    }

    const binding = try parser.allocator.create(stmt.LexicalBinding);
    binding.* = stmt.LexicalBinding{
        .name = name,
        .initializer = initializer,
    };

    return binding;
}

pub fn parse_if_statement(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.IfStatement {
    const token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;
    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (!is_keyword(data, "if")) {
        return error.UnexpectedEndOfTokens;
    }
    _ = p.next(parser);

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.OpenParen),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const test_expr = exp_parser.parse_expression(parser) catch return error.UnexpectedEndOfTokens;

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const consequent_stmt = try parse_statement(parser);

    const if_stmt = try parser.allocator.create(stmt.IfStatement);
    const maybe_stmt = try parser.allocator.create(stmt.MaybeStatement);

    maybe_stmt.* = .{
        .has_value = false,
        .value = .{
            .none = {},
        },
    };

    if_stmt.* = .{
        .test_ = &test_expr.*,
        .consequent = &consequent_stmt.*,
        .alternate = &maybe_stmt.*,
    };

    const next_token = p.peek(parser) orelse return if_stmt;
    if (next_token.kind == .CommonToken) {
        const next_data: *CommonTokenData = @ptrFromInt(next_token.data);
        if (is_keyword(next_data, "else")) {
            _ = p.next(parser);

            const alternate_stmt = try parse_statement(parser);

            maybe_stmt.* = stmt.MaybeStatement{
                .has_value = true,
                .value = .{
                    .value = alternate_stmt.*,
                },
            };

            if_stmt.*.alternate = maybe_stmt;
        }
    }

    // std.debug.print("Parsed if statement with test: {any}, consequent: {any}, alternate: {any}\n", .{
    //     test_expr.*,
    //     consequent_stmt.*,
    //     maybe_stmt.*,
    // });

    return if_stmt;
}

test "parse if statement (without else)" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "if (a) { let b; }");

    std.debug.assert(result.tag == stmt.STATEMENT_IF_STATEMENT);
    std.debug.assert(result.data.if_statement.test_.len == 1);
    std.debug.assert(result.data.if_statement.test_.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.test_.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));

    std.debug.assert(result.data.if_statement.consequent.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);

    std.debug.assert(result.data.if_statement.alternate.has_value == false);
}

test "parse if statement (with else)" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "if (a) { let b; } else { let c; }");

    std.debug.assert(result.tag == stmt.STATEMENT_IF_STATEMENT);
    std.debug.assert(result.data.if_statement.test_.len == 1);
    std.debug.assert(result.data.if_statement.test_.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.test_.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));

    std.debug.assert(result.data.if_statement.consequent.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);

    std.debug.assert(result.data.if_statement.alternate.has_value == true);
    std.debug.assert(result.data.if_statement.alternate.value.value.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("c")), 1),
    ));

    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

test "parse if statement (with else if)" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "if (a) { let b; } else if (c) { let d; }");

    std.debug.assert(result.tag == stmt.STATEMENT_IF_STATEMENT);
    std.debug.assert(result.data.if_statement.test_.len == 1);
    std.debug.assert(result.data.if_statement.test_.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.test_.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.test_.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));

    std.debug.assert(result.data.if_statement.consequent.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);

    std.debug.assert(result.data.if_statement.alternate.has_value == true);
    std.debug.assert(result.data.if_statement.alternate.value.value.tag == stmt.STATEMENT_IF_STATEMENT);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.test_.len == 1);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.test_.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.test_.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.test_.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.test_.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.alternate.value.value.data.if_statement.test_.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("c")), 1),
    ));

    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("d")), 1),
    ));
    std.debug.assert(result.data.if_statement.alternate.value.value.data.if_statement.consequent.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

pub fn parse_while_statement(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.WhileStatement {
    const token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;
    const data: *CommonTokenData = @ptrFromInt(token.data);

    if (!is_keyword(data, "while")) {
        return error.UnexpectedEndOfTokens;
    }
    _ = p.next(parser);

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.OpenParen),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const test_expr = exp_parser.parse_expression(parser) catch return error.UnexpectedEndOfTokens;

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const body_stmt = try parse_statement(parser);

    const while_stmt = try parser.allocator.create(stmt.WhileStatement);
    while_stmt.* = .{
        .test_ = &test_expr.*,
        .body = &body_stmt.*,
    };

    return while_stmt;
}

test "parse while statement" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "while (a) { let b; }");

    std.debug.assert(result.tag == stmt.STATEMENT_WHILE);
    std.debug.assert(result.data.while_.test_.len == 1);
    std.debug.assert(result.data.while_.test_.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.while_.test_.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.while_.test_.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.while_.test_.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.while_.test_.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));

    std.debug.assert(result.data.while_.body.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.while_.body.data.block_statement.body.len == 1);
    std.debug.assert(result.data.while_.body.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.while_.body.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.while_.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.while_.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.while_.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));
    std.debug.assert(result.data.while_.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

pub fn parse_do_while_statement(parser: *Parser) error{ UnexpectedEndOfTokens, OutOfMemory }!*stmt.WhileStatement {
    const do_token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;
    const do_data: *CommonTokenData = @ptrFromInt(do_token.data);
    if (!is_keyword(do_data, "do")) {
        return error.UnexpectedEndOfTokens;
    }
    _ = p.next(parser);

    const body_stmt = try parse_statement(parser);

    const while_token = p.peek(parser) orelse return error.UnexpectedEndOfTokens;
    const while_data: *CommonTokenData = @ptrFromInt(while_token.data);
    if (!is_keyword(while_data, "while")) {
        return error.UnexpectedEndOfTokens;
    }
    _ = p.next(parser);

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.OpenParen),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const test_expr = exp_parser.parse_expression(parser) catch return error.UnexpectedEndOfTokens;

    p.expect_skip_whitespace(parser, _text.Token{
        .kind = .CommonToken,
        .data = @intFromPtr(&CommonTokenData{
            .common_token_kind = .Punctuator,
            .data = @intFromEnum(_text.PunctuatorKind.CloseParen),
        }),
    }) catch return error.UnexpectedEndOfTokens;
    _ = p.next(parser);

    const while_stmt = try parser.allocator.create(stmt.WhileStatement);
    while_stmt.* = .{
        .test_ = &test_expr.*,
        .body = &body_stmt.*,
    };

    return while_stmt;
}

test "parse do while statement" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "do { let b; } while (a);");

    std.debug.assert(result.tag == stmt.STATEMENT_DO_WHILE);
    std.debug.assert(result.data.do_while.test_.len == 1);
    std.debug.assert(result.data.do_while.test_.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.do_while.test_.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.do_while.test_.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.do_while.test_.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.do_while.test_.data[0].data.lhs.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("a")), 1),
    ));

    std.debug.assert(result.data.do_while.body.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.do_while.body.data.block_statement.body.len == 1);
    std.debug.assert(result.data.do_while.body.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_DECLARATION);
    std.debug.assert(result.data.do_while.body.data.block_statement.body.data[0].data.declaration.tag == stmt.DECLARATION_LEXICAL_DECLARATION);
    std.debug.assert(result.data.do_while.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.is_const == false);
    std.debug.assert(result.data.do_while.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.len == 1);
    std.debug.assert(testing.are_equal_strings(
        result.data.do_while.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].name.name,
        testing.u8_array_to_string(@ptrCast(@constCast("b")), 1),
    ));

    std.debug.assert(result.data.do_while.body.data.block_statement.body.data[0].data.declaration.data.lexical_declaration.declarations.data[0].initializer.has_value == false);
}

test "parse example #1" {
    const result = try get_parse_result(stmt.Statement, parse_statement, "if (x > 0) { console.log('positive'); } else { console.log('non-positive'); }");

    std.debug.assert(result.tag == stmt.STATEMENT_IF_STATEMENT);
    std.debug.assert(result.data.if_statement.test_.len == 1);
    std.debug.assert(result.data.if_statement.test_.data[0].tag == expr.ASSIGNMENT_EXPR_BINARY);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.operator == .GreaterThan);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.left.tag == expr.BINARY_OR_UNARY_UNARY);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.left.data.unary.operator == .None);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.left.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.left.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.left.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.test_.data[0].data.binary.left.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("x")), 1),
    ));

    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.tag == expr.UNARY_EXPR_OR_NULL_UNARY);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operator == .None);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.tag == expr.UNARY_EXPR_OR_LHS_LHS);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.data.left_hand_side.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.if_statement.test_.data[0].data.binary.right.data.unary.operand.data.left_hand_side.data.new.data.member.data.primary.data.literal.data.number.value == 0);

    std.debug.assert(result.data.if_statement.consequent.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_STATEMENT);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.tag == stmt.STATEMENT_EXPR_STATEMENT);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.len == 1);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_CALL);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.tag == expr.CALL_EXPR_COVER);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.data.property.object.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("console")), 7),
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.data.property.property.name,
        testing.u8_array_to_string(@ptrCast(@constCast("log")), 3),
    ));

    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.len == 1);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_STRING);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.consequent.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.literal.data.string.*,
        testing.u8_array_to_string(@ptrCast(@constCast("positive")), 8),
    ));

    std.debug.assert(result.data.if_statement.alternate.has_value == true);
    std.debug.assert(result.data.if_statement.alternate.value.value.tag == stmt.STATEMENT_BLOCK_STATEMENT);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.len == 1);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].tag == stmt.STATEMENT_OR_DECLARATION_STATEMENT);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.tag == stmt.STATEMENT_EXPR_STATEMENT);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.len == 1);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_CALL);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.tag == expr.CALL_EXPR_COVER);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.tag == expr.MEMBER_EXPR_PROPERTY);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.data.property.object.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.data.property.object.data.primary.data.identifier.data.identifier.name,
        testing.u8_array_to_string(@ptrCast(@constCast("console")), 7),
    ));

    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.callee.data.property.property.name,
        testing.u8_array_to_string(@ptrCast(@constCast("log")), 3),
    ));

    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.len == 1);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].tag == expr.ASSIGNMENT_EXPR_LHS);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.tag == expr.LEFT_HAND_SIDE_EXPR_NEW);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.tag == expr.NEW_EXPR_MEMBER);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.tag == expr.MEMBER_EXPR_PRIMARY);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.literal.tag == expr.LITERAL_STRING);
    std.debug.assert(testing.are_equal_strings(
        result.data.if_statement.alternate.value.value.data.block_statement.body.data[0].data.statement.data.expr_statement.data[0].data.lhs.data.call.data.cover.arguments.arguments.data[0].data.lhs.data.new.data.member.data.primary.data.literal.data.string.*,
        testing.u8_array_to_string(@ptrCast(@constCast("non-positive")), 12),
    ));
}
