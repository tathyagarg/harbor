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

    while (true) {
        const next_token = peek(parser) orelse break;

        if (next_token.kind == .Whitespace or next_token.kind == .LineTerminator) {
            parser.curr += 1;
            continue;
        }

        break;
    }

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
        const token = peek(parser) orelse break;

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
