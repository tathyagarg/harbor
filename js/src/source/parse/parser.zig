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
    const token = next(parser) orelse return error.UnexpectedEndOfTokens;
    if (token != expected) {
        return error.UnexpectedToken;
    }
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

pub fn parse_primary_expression(parser: *Parser) !expr.PrimaryExpression {
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

    return primary_expr.*;
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

    // var parser = init(TokenSeq{
    //     .data = &tokens,
    //     .len = 1,
    // });

    const result = try parse_primary_expression(&parser);

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

    const result = try parse_primary_expression(&parser);

    std.debug.assert(result.tag == expr.PRIMARY_EXPR_LITERAL);
    std.debug.assert(result.data.literal.tag == expr.LITERAL_NUMBER);
    std.debug.assert(result.data.literal.data.number.value == 42.0);

    deinit(&parser);
}
