const std = @import("std");

pub const testing = @import("testing.zig");

const source_text = @import("source/text.zig");
const expr = @import("source/parse/expressions.zig");
const exp_parser = @import("source/parse/exp_parser.zig");
const stmt = @import("source/parse/statements.zig");
const stmt_parser = @import("source/parse/stmt_parser.zig");
const p = @import("source/parse/parser.zig");

test {
    _ = @import("source/parse/exp_parser.zig");
}

test {
    _ = @import("source/parse/stmt_parser.zig");
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

pub export fn parse_text(text: source_text.String) stmt.Script {
    const tokens = parse_text_string(text, .InputElementHashbangOrRegExp);

    var parser = p.Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    std.debug.print("Starting to parse script\n", .{});

    const result = stmt_parser.parse_text(&parser) catch {
        const script = parser.allocator.create(stmt.Script) catch {
            std.debug.print("Failed to allocate script\n", .{});
            return stmt.Script{
                .body = .{
                    .data = &[_]stmt.StatementOrDeclaration{},
                    .len = 0,
                },
            };
        };
        const seq = parser.allocator.create(source_text.Seq(stmt.StatementOrDeclaration)) catch {
            std.debug.print("Failed to allocate statement sequence\n", .{});
            return stmt.Script{
                .body = .{
                    .data = &[_]stmt.StatementOrDeclaration{},
                    .len = 0,
                },
            };
        };

        seq.* = .{
            .data = &[_]stmt.StatementOrDeclaration{},
            .len = 0,
        };

        script.* = .{
            .body = seq.*,
        };

        return script.*;
    };

    std.debug.print("Parsed script successfully: {d} statements\n", .{result.body.len});

    return result.*;
}
