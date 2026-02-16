const std = @import("std");

pub const testing = @import("testing.zig");

const source_text = @import("source/text.zig");
const expr = @import("source/parse/expressions.zig");
const _parser = @import("source/parse/parser.zig");

test {
    _ = @import("source/parse/parser.zig");
}

test {
    _ = @import("source/numeral.zig");
}

test {
    _ = @import("source/text.zig");
}

pub const CodePoint = u32;
pub const SourceCharacter = CodePoint;

pub export fn utf16_encode_cp(cp: CodePoint) source_text.String {
    return source_text.utf16_encode_cp(cp);
}

pub export fn cps_to_string(text: [*]CodePoint, len: usize) source_text.String {
    return source_text.cps_to_string(text, len) catch
        source_text.String{
            .data = &[_]u16{},
            .len = 0,
        };
}

pub export fn utf16_surrogate_pair_to_cp(high: u16, low: u16) CodePoint {
    return source_text.utf16_surrogate_pair_to_cp(high, low);
}

pub export fn code_point_at(text: source_text.String, position: usize) source_text.CodePointAtResult {
    return source_text.code_point_at(text, position);
}

pub export fn string_to_cps(text: source_text.String) source_text.CodePointSeq {
    return source_text.string_to_cps(text) catch source_text.CodePointSeq{
        .data = &[_]CodePoint{},
        .len = 0,
    };
}

pub export fn parse_text_string(text: source_text.String, goal: source_text.GoalSymbol) source_text.TokenSeq {
    return source_text.parse_text_string(text, goal) catch source_text.TokenSeq{
        .data = &[_]source_text.Token{},
        .len = 0,
    };
}

pub export fn parse_text_cps(cps: source_text.CodePointSeq, goal: source_text.GoalSymbol) source_text.TokenSeq {
    return source_text.parse_text_cps(cps, goal) catch source_text.TokenSeq{
        .data = &[_]source_text.Token{},
        .len = 0,
    };
}

pub export fn free_string(str: source_text.String) void {
    source_text.free_string(str);
}

pub export fn free_code_point_seq(seq: source_text.CodePointSeq) void {
    source_text.free_code_point_seq(seq);
}

pub export fn free_token_seq(seq: source_text.TokenSeq) void {
    source_text.free_token_seq(seq);
}

pub export fn temp_unsafe_parse_primary_expr(tokens: source_text.TokenSeq) expr.PrimaryExpression {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);

    var parser = _parser.Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = arena,
        .allocator = arena.allocator(),
    };

    const res = _parser.parse_primary_expression(&parser) catch |err| {
        // panic
        std.debug.print("Error parsing primary expression: {any}\n", .{err});
        return expr.PrimaryExpression{
            .tag = 0,
            .data = .{ .this = {} },
        };
    };

    return res;
}
