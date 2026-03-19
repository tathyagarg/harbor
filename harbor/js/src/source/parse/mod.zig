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
