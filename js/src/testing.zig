const std = @import("std");

const String = @import("source/text.zig").String;

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
