const std = @import("std");

const lex = @import("source/text.zig");
const String = @import("source/text.zig").String;
const Token = @import("source/text.zig").Token;

pub fn u8_array_to_string(text: [*]u8, len: usize) String {
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

pub fn are_equal_strings(actual: String, expected: String) bool {
    if (actual.len != expected.len) {
        return false;
    }

    for (actual.data, 0..actual.len) |a, i| {
        if (a != expected.data[i]) {
            return false;
        }
    }

    return true;
}

pub fn are_equal_strings_raw(actual: String, expected: [*]u8, expected_len: usize) bool {
    if (actual.len != expected_len) {
        return false;
    }

    for (actual.data, 0..actual.len) |a, i| {
        if (a != @as(u16, @intCast(expected[i]))) {
            return false;
        }
    }

    return true;
}

pub fn are_equal_strings_pure(actual: String, expected: []const u8) bool {
    if (actual.len != expected.len) {
        return false;
    }

    for (actual.data, 0..actual.len) |a, i| {
        if (a != @as(u16, @intCast(expected[i]))) {
            return false;
        }
    }

    return true;
}

pub fn are_equal_tokens(actual: Token, expected: Token) bool {
    if (actual.kind != expected.kind) {
        return false;
    }

    switch (actual.kind) {
        .CommonToken => {
            const actual_token: *lex.CommonTokenData = @ptrFromInt(actual.data);
            const expected_token: *lex.CommonTokenData = @ptrFromInt(expected.data);

            if (actual_token.common_token_kind != expected_token.common_token_kind) {
                return false;
            }

            switch (actual_token.common_token_kind) {
                .IdentifierName => {
                    const actual_identifier_name: *lex.IdentifierNameData = @ptrFromInt(actual_token.data);
                    const expected_identifier_name: *lex.IdentifierNameData = @ptrFromInt(expected_token.data);

                    return are_equal_strings(actual_identifier_name.name, expected_identifier_name.name);
                },
                .StringLiteral => {
                    const actual_string_literal: *lex.StringLiteralData = @ptrFromInt(actual_token.data);
                    const expected_string_literal: *lex.StringLiteralData = @ptrFromInt(expected_token.data);

                    return are_equal_strings(actual_string_literal.value, expected_string_literal.value);
                },
                .NumericLiteral => {
                    const actual_numeric_literal: *lex.NumericLiteralData = @ptrFromInt(actual_token.data);
                    const expected_numeric_literal: *lex.NumericLiteralData = @ptrFromInt(expected_token.data);

                    return actual_numeric_literal.value == expected_numeric_literal.value;
                },
                .Punctuator => {
                    const actual_punc_kind: lex.PunctuatorKind = @enumFromInt(actual_token.data);
                    const expected_punc_kind: lex.PunctuatorKind = @enumFromInt(expected_token.data);

                    return actual_punc_kind == expected_punc_kind;
                },
                else => {
                    return false;
                },
            }
        },
        else => {
            return false;
        },
    }
}

pub fn print_string(str: String) void {
    std.debug.print("String(len={d}): ", .{str.len});
    for (str.data, 0..str.len) |c, _| {
        std.debug.print("{c}", .{@as(u8, @intCast(c))});
    }
}
