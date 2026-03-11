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
