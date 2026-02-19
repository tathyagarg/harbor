const std = @import("std");
const root = @import("../root.zig");
const unicode = @import("unicode.zig");
const testing = @import("../testing.zig");

const CodePointSeq = @import("text.zig").CodePointSeq;
const CommonTokenData = @import("text.zig").CommonTokenData;
const NumericLiteralData = @import("text.zig").NumericLiteralData;
const display_code_point_seq = @import("text.zig").display_code_point_seq;
const Token = @import("text.zig").Token;

const ZERO = 0x0030;
const UNDERSCORE = 0x005F;

pub fn match_numeric_literal(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?Token {
    const original_i = i.*;

    if (match_decimal_literal(text, i, cp)) |token| {
        return token;
    }

    i.* = original_i;
    std.debug.print("Failed to match numeric literal at index {d}\n", .{original_i});
    return null;
}

test "simple decimal literal" {
    const str = "0";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_literal(cps, &i, cps.data[i]);
    std.debug.assert(result != null);

    const definite_result = result.?;

    std.debug.assert(definite_result.kind == .CommonToken);

    const common_data: *const CommonTokenData = @ptrFromInt(definite_result.data);
    const numeric_data: *const NumericLiteralData = @ptrFromInt(common_data.*.data);

    std.debug.assert(numeric_data.*.value == 0);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

test "decimal literal with underscores" {
    const str = "1_000_000";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_literal(cps, &i, cps.data[i]);
    std.debug.assert(result != null);

    const definite_result = result.?;

    std.debug.assert(definite_result.kind == .CommonToken);

    const common_data: *const CommonTokenData = @ptrFromInt(definite_result.data);
    const numeric_data: *const NumericLiteralData = @ptrFromInt(common_data.*.data);

    std.debug.assert(numeric_data.*.value == 1000000);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

test "123.456" {
    const str = "123.456";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_literal(cps, &i, cps.data[i]);
    std.debug.assert(result != null);

    const definite_result = result.?;

    std.debug.assert(definite_result.kind == .CommonToken);

    const common_data: *const CommonTokenData = @ptrFromInt(definite_result.data);
    const numeric_data: *const NumericLiteralData = @ptrFromInt(common_data.*.data);

    std.debug.assert(numeric_data.*.value == 123.456);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

test "1.25" {
    const str = "1.25";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_literal(cps, &i, cps.data[i]);
    std.debug.assert(result != null);

    const definite_result = result.?;

    std.debug.assert(definite_result.kind == .CommonToken);

    const common_data: *const CommonTokenData = @ptrFromInt(definite_result.data);
    const numeric_data: *const NumericLiteralData = @ptrFromInt(common_data.*.data);

    std.debug.assert(numeric_data.*.value == 1.25);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

fn match_decimal_literal(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?Token {
    if (cp == ZERO) {
        i.* += 1;

        const common_token_data = std.heap.page_allocator.create(CommonTokenData) catch {
            std.debug.print("Failed to create common token data\n", .{});
            return null;
        };

        const numeric_literal_data = std.heap.page_allocator.create(NumericLiteralData) catch {
            std.debug.print("Failed to create numeric literal data\n", .{});
            return null;
        };

        numeric_literal_data.* = NumericLiteralData{
            .value = 0,
            .is_bigint = false,
            .number_system = .Decimal,
        };

        common_token_data.* = CommonTokenData{
            .common_token_kind = .NumericLiteral,
            .data = @intFromPtr(numeric_literal_data),
        };

        return Token{
            .kind = .CommonToken,
            .data = @intFromPtr(common_token_data),
        };
    }

    if (match_decimal_integer_literal(text, i, cp)) |value| {
        const common_token_data = std.heap.page_allocator.create(CommonTokenData) catch {
            std.debug.print("Failed to create common token data\n", .{});
            return null;
        };

        const numeric_literal_data = std.heap.page_allocator.create(NumericLiteralData) catch {
            std.debug.print("Failed to create numeric literal data\n", .{});
            return null;
        };

        numeric_literal_data.* = NumericLiteralData{
            .value = 0,
            .is_bigint = false,
            .number_system = .Decimal,
        };

        common_token_data.* = CommonTokenData{
            .common_token_kind = .NumericLiteral,
            .data = @intFromPtr(numeric_literal_data),
        };

        if (text.data[i.*] == '.') {
            i.* += 1;

            if (match_decimal_digits(text, i, true)) |fractional| {
                const result = @as(f64, @floatFromInt(value)) +
                    (@as(f64, @floatFromInt(fractional)) / @as(f64, @floatFromInt(std.math.pow(u64, 10, std.math.log10(fractional) + 1))));

                numeric_literal_data.*.value = result;

                return Token{
                    .kind = .CommonToken,
                    .data = @intFromPtr(common_token_data),
                };
            } else {
                numeric_literal_data.*.value = @as(f64, @floatFromInt(value));

                return Token{
                    .kind = .CommonToken,
                    .data = @intFromPtr(common_token_data),
                };
            }
        } else {
            numeric_literal_data.*.value = @as(f64, @floatFromInt(value));

            return Token{
                .kind = .CommonToken,
                .data = @intFromPtr(common_token_data),
            };
        }
    }

    return null;
}

test "decimal literal with fractional part" {
    const str = "123.456";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_literal(cps, &i, cps.data[i]);
    std.debug.assert(result != null);

    const definite_result = result.?;

    std.debug.assert(definite_result.kind == .CommonToken);

    const common_data: *const CommonTokenData = @ptrFromInt(definite_result.data);
    const numeric_data: *const NumericLiteralData = @ptrFromInt(common_data.*.data);

    std.debug.assert(numeric_data.*.value == 123.456);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

test "decimal literal without fractional part" {
    const str = "123";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_literal(cps, &i, cps.data[i]);
    std.debug.assert(result != null);

    const definite_result = result.?;

    std.debug.assert(definite_result.kind == .CommonToken);

    const common_data: *const CommonTokenData = @ptrFromInt(definite_result.data);
    const numeric_data: *const NumericLiteralData = @ptrFromInt(common_data.*.data);

    std.debug.assert(numeric_data.*.value == 123);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

fn match_decimal_integer_literal(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?u64 {
    const original_i = i.*;

    if (cp == ZERO) {
        if (match_non_octal_decimal_integer_literal(text, i)) |value| {
            return value;
        }

        i.* += 1;
        return 0;
    }

    // unicode.is_decimal_digit checks for 0-9, but we only want 1-9
    // Since cp == ZERO has already been handled, this still works
    if (unicode.is_decimal_digit(cp)) {
        i.* += 1;
        var value: u64 = @intCast(cp - ZERO);

        if (text.data[i.*] == UNDERSCORE) {
            i.* -= 1;

            if (match_decimal_digits(text, i, true)) |res| {
                return res;
            } else {
                i.* = original_i;

                std.debug.print("Expected digits after underscore in numeric literal\n", .{});
                return null;
            }
        } else if (match_decimal_digits(text, i, true)) |res| {
            value = (value * std.math.pow(u64, 10, (std.math.log10(res) + 1))) + res;
            return value;
        } else {
            return value;
        }
    }

    if (match_non_octal_decimal_integer_literal(text, i)) |value| {
        return value;
    }

    i.* = original_i;
    return null;
}

test "regular decimal integer literal" {
    const str = "123456789";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_integer_literal(cps, &i, cps.data[i]) orelse 0;
    std.debug.assert(result == 123456789);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

fn match_non_octal_decimal_integer_literal(text: CodePointSeq, i: *usize) ?u64 {
    var res: u64 = 0;

    while (i.* < text.len) {
        const this_cp = text.data[i.*];

        const next = if (i.* + 1 < text.len) text.data[i.* + 1] else 0;

        if (this_cp == ZERO and unicode.is_non_octal_digit(next)) {
            i.* += 2;

            res = res * 10 + (next - ZERO);

            while (i.* < text.len) {
                if (unicode.is_decimal_digit(text.data[i.*])) {
                    res = res * 10 + (text.data[i.*] - ZERO);
                    i.* += 1;
                } else {
                    break;
                }
            }

            return res;
        } else if (match_legacy_octal_like_decimal_integer_literal(text, i, this_cp)) |value| {
            if (unicode.is_non_octal_digit(text.data[i.*])) {
                const this = text.data[i.*] - ZERO;
                const decimal = decimal_to_octal(value) * 10 + this;

                res = res * std.math.pow(u64, 10, std.math.log10(decimal) + 1) + decimal;
                i.* += 1;

                while (i.* < text.len) {
                    if (unicode.is_decimal_digit(text.data[i.*])) {
                        res = res * 10 + (text.data[i.*] - ZERO);
                        i.* += 1;
                    } else {
                        break;
                    }
                }

                return res;
            } else if (unicode.is_decimal_digit(text.data[i.*])) {
                res = res * 10 + (text.data[i.*] - ZERO);
                i.* += 1;

                while (i.* < text.len) {
                    if (unicode.is_decimal_digit(text.data[i.*])) {
                        res = res * 10 + (text.data[i.*] - ZERO);
                        i.* += 1;
                    } else {
                        break;
                    }
                }

                return res;
            } else {
                std.debug.print("Expected non-octal digit after octal-like decimal integer literal\n", .{});
                return null;
            }
        } else {
            break;
        }

        if (unicode.is_decimal_digit(text.data[i.*])) {
            res = res * 10 + (text.data[i.*] - ZERO);
            i.* += 1;

            while (i.* < text.len) {
                if (unicode.is_decimal_digit(text.data[i.*])) {
                    res = res * 10 + (text.data[i.*] - ZERO);
                    i.* += 1;
                } else {
                    break;
                }
            }

            return res;
        }
    }

    return null;
}

test "non-octal decimal integer literal starting with legacy octal-like prefix" {
    const str = "0789";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_integer_literal(cps, &i, cps.data[i]) orelse 0;
    std.debug.assert(result == 789);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

test "longer non-octal decimal integer literal starting with legacy octal-like prefix" {
    const str = "0123456789";

    const text = testing.u8_array_to_string(@ptrCast(@constCast(str)), str.len);
    const cps = root.string_to_cps(text);

    var i: usize = 0;

    const result = match_decimal_integer_literal(cps, &i, cps.data[i]) orelse 0;
    std.debug.assert(result == 123456789);

    root.free_string(text);
    root.free_code_point_seq(cps);
}

fn match_legacy_octal_like_decimal_integer_literal(text: CodePointSeq, i: *usize, cp: root.CodePoint) ?u64 {
    const next = if (i.* + 1 < text.len) text.data[i.* + 1] else 0;

    if (cp == ZERO and unicode.is_octal_digit(next)) {
        i.* += 2;

        var value: u64 = @intCast(next - ZERO);

        while (i.* < text.len) {
            const new_cp = text.data[i.*];

            if (unicode.is_octal_digit(new_cp)) {
                value = value * 8 + (new_cp - ZERO);
                i.* += 1;
            } else {
                break;
            }
        }

        return value;
    }

    return null;
}

fn match_decimal_digits(text: CodePointSeq, i: *usize, sep: bool) ?u64 {
    if (i.* >= text.len or !unicode.is_decimal_digit(text.data[i.*])) {
        return null;
    }

    var res: u64 = 0;
    var last_was_sep = false;

    while (i.* < text.len) {
        if (unicode.is_decimal_digit(text.data[i.*])) {
            last_was_sep = false;
            i.* += 1;
            res = res * 10 + (text.data[i.* - 1] - ZERO);
        } else if (sep and text.data[i.*] == UNDERSCORE) {
            if (last_was_sep) {
                std.debug.print("Consecutive underscores are not allowed in numeric literals\n", .{});
                return null;
            }

            last_was_sep = true;
            i.* += 1;
        } else {
            break;
        }
    }

    if (last_was_sep) {
        std.debug.print("Numeric literals cannot end with an underscore\n", .{});
        return null;
    }

    return res;
}

fn octal_to_decimal(octal: u64) u64 {
    var oct = octal;

    var decimal: u64 = 0;
    var multiplier: u64 = 1;

    while (oct > 0) {
        const digit = oct % 10;
        decimal += digit * multiplier;
        multiplier *= 8;
        oct /= 10;
    }

    return decimal;
}

fn decimal_to_octal(decimal: u64) u64 {
    var dec = decimal;

    var octal: u64 = 0;
    var multiplier: u64 = 1;

    while (dec > 0) {
        const digit = dec % 8;
        octal += digit * multiplier;
        multiplier *= 10;
        dec /= 8;
    }

    return octal;
}
