const std = @import("std");
const testing = @import("../../testing.zig");

const _text = @import("../text.zig");
const Parser = @import("./parser.zig").Parser;

pub fn EXTERN_UNION(comptime T: type) type {
    return extern struct {
        tag: u8,
        data: T,
    };
}

pub fn MAYBE(comptime T: type) type {
    return extern struct {
        has_value: bool,
        value: extern union {
            value: T,
            none: void,
        },
    };
}

pub const MAYBE_NONE = 0;
pub const MAYBE_VALUE = 1;

pub fn get_parse_result(
    T: type,
    f: fn (*Parser) error{ OutOfMemory, UnexpectedToken, UnexpectedEndOfTokens }!*T,
    text: []const u8,
) !T {
    const str = testing.u8_array_to_string(@ptrCast(@constCast(text)), text.len);

    const tokens = try _text.parse_text_string(str, .InputElementHashbangOrRegExp);

    var parser = Parser{
        .tokens = tokens,
        .curr = 0,
        .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        .allocator = undefined,
    };
    parser.allocator = parser.arena.allocator();

    const result = try f(&parser);

    return result.*;
}
