const String = @import("../text.zig").String;
const NumericLiteralData = @import("../text.zig").NumericLiteralData;
const IdentifierNameData = @import("../text.zig").IdentifierNameData;

pub fn EXTERN_UNION(comptime T: type) type {
    return extern struct {
        tag: u8,
        data: T,
    };
}

pub const Expression = extern struct {
    ptr: [*]AssignmentExpression,
    len: usize,
};

pub const PrimaryExpression = EXTERN_UNION(
    extern union {
        this: *void,
        identifier: *IdentifierReference,
        literal: *Literal,
        array: *ArrayLiteral,
        object: *ObjectLiteral,
    },
);

pub const YieldExpression = EXTERN_UNION(
    extern union {
        empty: *void,
        yield: *AssignmentExpression,
        yield_star: *AssignmentExpression,
    },
);

pub const LeftHandSideExpression = EXTERN_UNION(
    extern union {
        new: *NewExpression,
        call: *CallExpression,
        optional: *OptionalExpression,
    },
);

pub const OptionalExpression = EXTERN_UNION(
    extern struct {
        kind: extern union {
            member: *MemberExpression,
            call: *CallExpression,
        },
        chains: []*OptionalChain,
    },
);

pub const OptionalChain = EXTERN_UNION(
    extern union {
        args: *Arguments,
        member: *Expression,
        property: *IdentifierNameData,
        private_property: *IdentifierNameData,
        chain_args: extern struct {
            chain: *OptionalChain,
            arguments: *Arguments,
        },
        chain_member: extern struct {
            chain: *OptionalChain,
            expr: *Expression,
        },
        chain_property: extern struct {
            chain: *OptionalChain,
            property: *IdentifierNameData,
        },
        chain_private_property: extern struct {
            chain: *OptionalChain,
            property: *IdentifierNameData,
        },
    },
);

pub const Arguments = extern struct {
    arguments: []*AssignmentExpression,
};

pub const SuperCall = Arguments;

pub const ImportCall = extern struct {
    ptr: [*]AssignmentExpression,
    len: usize,
};

pub const CallExpression = EXTERN_UNION(
    extern union {
        super: *SuperCall,
        import: *ImportCall,
        simple_call: extern struct {
            callee: *CallExpression,
            arguments: *Arguments,
        },
        member: extern struct {
            object: *CallExpression,
            expr: *Expression,
        },
        property: extern struct {
            object: *CallExpression,
            property: *IdentifierNameData,
        },
        private_property: extern struct {
            object: *CallExpression,
            property: *IdentifierNameData,
        },
    },
);

pub const MemberExpression = EXTERN_UNION(
    extern union {
        primary: *PrimaryExpression,
        member: extern struct {
            object: *MemberExpression,
            expr: *Expression,
        },
        property: extern struct {
            object: *MemberExpression,
            property: *IdentifierNameData,
        },
        super: *SuperProperty,
        import_meta: *MetaProperty,
        new: extern struct {
            callee: *MemberExpression,
            arguments: *Arguments,
        },
        private_property: extern struct {
            object: *MemberExpression,
            property: *IdentifierNameData,
        },
    },
);

pub const SuperProperty = EXTERN_UNION(
    extern union {
        member: *Expression,
        property: *IdentifierNameData,
    },
);

pub const MetaProperty = enum(u8) {
    NewTarget = 0,
    ImportMeta = 1,
};

pub const NewExpression = EXTERN_UNION(extern union {
    member: *MemberExpression,
    new: *NewExpression,
});

pub const AssignmentExpression = EXTERN_UNION(
    extern union {
        conditional: *ConditionalExpression,
        yield: *YieldExpression,
        raw_assignment: extern struct {
            left: *LeftHandSideExpression,
            right: *AssignmentExpression,
        },
        operator_assignment: extern struct {
            left: *LeftHandSideExpression,
            operator: *AssignmentOperator,
            right: *AssignmentExpression,
        },
    },
);

pub const AssignmentOperator = enum(u8) {
    Star = 0,
    Slash = 1,
    Percent = 2,
    Plus = 3,
    Minus = 4,
    LeftShift = 5,
    RightShift = 6,
    UnsignedRightShift = 7,
    BitwiseAnd = 8,
    BitwiseXor = 9,
    BitwiseOr = 10,
    Exponentiation = 11,
    ShortCircuitLogicalAnd = 12,
    ShortCircuitLogicalOr = 13,
    NullishCoalescing = 14,
};

pub const ConditionalExpression = EXTERN_UNION(
    extern union {
        short_circuit: *ShortCircuitExpression,
        conditional: extern struct {
            short_circuit: *ShortCircuitExpression,
            consequent: *AssignmentExpression,
            alternate: *AssignmentExpression,
        },
    },
);

pub const ShortCircuitExpression = EXTERN_UNION(
    extern union {
        logical_or: *LogicalORExpression,
        nullish_coalescing: *void,
    },
);

pub const LogicalORExpression = EXTERN_UNION(
    extern union {
        logical_and: *LogicalANDExpression,
        logical_or: extern struct {
            left: *LogicalORExpression,
            right: *LogicalANDExpression,
        },
    },
);

pub const LogicalANDExpression = EXTERN_UNION(
    extern union {
        bitwise_or: *BitwiseORExpression,
        logical_and: extern struct {
            left: *LogicalANDExpression,
            right: *BitwiseORExpression,
        },
    },
);

pub const BitwiseORExpression = EXTERN_UNION(
    extern union {
        bitwise_xor: *BitwiseXORExpression,
        bitwise_or: extern struct {
            left: *BitwiseORExpression,
            right: *BitwiseXORExpression,
        },
    },
);

pub const BitwiseXORExpression = EXTERN_UNION(
    extern union {
        bitwise_and: *BitwiseANDExpression,
        bitwise_xor: extern struct {
            left: *BitwiseXORExpression,
            right: *BitwiseANDExpression,
        },
    },
);

pub const BitwiseANDExpression = EXTERN_UNION(
    extern union {
        equality: *EqualityExpression,
        bitwise_and: extern struct {
            left: *BitwiseANDExpression,
            right: *EqualityExpression,
        },
    },
);

pub const EqualityExpression = EXTERN_UNION(
    extern union {
        relational: *RelationalExpression,
        equality: extern struct {
            left: *EqualityExpression,
            operator: *EqualityOperator,
            right: *RelationalExpression,
        },
    },
);

pub const EqualityOperator = enum(u8) {
    Equal = 0,
    NotEqual = 1,
    StrictEqual = 2,
    StrictNotEqual = 3,
};

pub const RelationalExpression = EXTERN_UNION(
    extern union {
        shift: *ShiftExpression,
        relational: extern struct {
            left: *RelationalExpression,
            operator: *RelationalOperator,
            right: *ShiftExpression,
        },
        private_identifier_in: extern struct {
            left: *IdentifierNameData,
            right: *ShiftExpression,
        },
    },
);

pub const RelationalOperator = enum(u8) {
    LessThan = 0,
    GreaterThan = 1,
    LessThanOrEqual = 2,
    GreaterThanOrEqual = 3,
    InstanceOf = 5,
    In = 4,
};

pub const ShiftExpression = EXTERN_UNION(
    extern union {
        additive: *AdditiveExpression,
        shift: extern struct {
            left: *ShiftExpression,
            operator: *ShiftOperator,
            right: *AdditiveExpression,
        },
    },
);

pub const ShiftOperator = enum(u8) {
    LeftShift = 0,
    RightShift = 1,
    UnsignedRightShift = 2,
};

pub const AdditiveExpression = EXTERN_UNION(
    extern union {
        multiplicative: *MultiplicativeExpression,
        additive: extern struct {
            left: *AdditiveExpression,
            operator: *AdditiveOperator,
            right: *MultiplicativeExpression,
        },
    },
);

pub const AdditiveOperator = enum(u8) {
    Plus = 0,
    Minus = 1,
};

pub const MultiplicativeExpression = EXTERN_UNION(
    extern union {
        exponentiation: *ExponentiationExpression,
        multiplicative: extern struct {
            left: *MultiplicativeExpression,
            operator: *MultiplicativeOperator,
            right: *ExponentiationExpression,
        },
    },
);

pub const MultiplicativeOperator = enum(u8) {
    Star = 0,
    Slash = 1,
    Percent = 2,
};

pub const ExponentiationExpression = EXTERN_UNION(
    extern union {
        unary: *UnaryExpression,
        exponentiation: extern struct {
            left: *UnaryExpression,
            right: *ExponentiationExpression,
        },
    },
);

pub const UnaryExpression = EXTERN_UNION(
    extern union {
        update: *UpdateExpression,
        unary: extern struct {
            operator: *UnaryOperator,
            operand: *UnaryExpression,
        },
        await: *AwaitExpression,
    },
);

pub const UnaryOperator = enum(u8) {
    Delete = 0,
    Void = 1,
    TypeOf = 2,
    Plus = 3,
    Minus = 4,
    BitwiseNot = 5,
    LogicalNot = 6,
};

pub const AwaitExpression = extern struct {
    expression: *UnaryExpression,
};

pub const UpdateExpression = EXTERN_UNION(
    extern union {
        left_hand_side: *LeftHandSideExpression,
        update: extern struct {
            operand: *LeftHandSideExpression,
            operator: *UpdateOperator,
            prefix: bool,
        },
    },
);

pub const UpdateOperator = enum(u8) {
    Increment = 0,
    Decrement = 1,
};

pub const IdentifierReference = EXTERN_UNION(
    extern union {
        identifier: *String,
        yield: *void,
        await: *void,
    },
);

pub const BindingIdentifier = EXTERN_UNION(
    extern union {
        identifier: *String,
        yield: *void,
        await: *void,
    },
);

pub const Literal = EXTERN_UNION(
    extern union {
        null: *void,
        boolean: *bool,
        string: *String,
        number: *NumericLiteralData,
    },
);

pub const ArrayLiteral = extern struct {
    elements: []*ArrayElement,
};

pub const ArrayElement = EXTERN_UNION(
    extern union {
        expression: *AssignmentExpression,
        spread: *AssignmentExpression,
        ellision: void,
    },
);

pub const ObjectLiteral = extern struct {
    properties: []*PropertyDefinition,
};

pub const PropertyDefinition = EXTERN_UNION(
    extern union {
        identifier: *IdentifierReference,
        cover_initialized_name: *CoverInitializedName,
        property: *Property,
        method: void, // TODO: MethodDefinition,
        spread: *AssignmentExpression,
    },
);

pub const CoverInitializedName = extern struct {
    identifier: *IdentifierReference,
    initializer: *AssignmentExpression,
};

pub const PropertyName = EXTERN_UNION(
    extern union {
        identifier_name: *String,
        string_literal: *String,
        numeric_literal: *NumericLiteralData,
        computed: *AssignmentExpression,
    },
);

pub const Property = extern struct {
    key: *PropertyName,
    value: *AssignmentExpression,
};
