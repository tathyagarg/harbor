pub const String = @import("../text.zig").String;
pub const NumericLiteralData = @import("../text.zig").NumericLiteralData;
pub const IdentifierNameData = @import("../text.zig").IdentifierNameData;

const Seq = @import("../text.zig").Seq;

pub fn EXTERN_UNION(comptime T: type) type {
    return extern struct {
        tag: u8,
        data: T,
    };
}

pub const Expression = Seq(AssignmentExpression);

pub const PrimaryExpression = EXTERN_UNION(
    extern union {
        this: *void,
        identifier: *IdentifierReference,
        literal: *Literal,
        array: *ArrayLiteral,
        object: *ObjectLiteral,
    },
);

pub const PRIMARY_EXPR_THIS = 0;
pub const PRIMARY_EXPR_IDENTIFIER = 1;
pub const PRIMARY_EXPR_LITERAL = 2;
pub const PRIMARY_EXPR_ARRAY = 3;
pub const PRIMARY_EXPR_OBJECT = 4;

pub const YieldExpression = EXTERN_UNION(
    extern union {
        empty: *void,
        yield: *AssignmentExpression,
        yield_star: *AssignmentExpression,
    },
);

pub const YIELD_EXPR_EMPTY = 0;
pub const YIELD_EXPR_YIELD = 1;
pub const YIELD_EXPR_YIELD_STAR = 2;

pub const LeftHandSideExpression = EXTERN_UNION(
    extern union {
        new: *NewExpression,
        call: *CallExpression,
        optional: *OptionalExpression,
    },
);

pub const LEFT_HAND_SIDE_EXPR_NEW = 0;
pub const LEFT_HAND_SIDE_EXPR_CALL = 1;
pub const LEFT_HAND_SIDE_EXPR_OPTIONAL = 2;

pub const OptionalExpression = EXTERN_UNION(
    extern struct {
        kind: extern union {
            member: *MemberExpression,
            call: *CallExpression,
        },
        chains: Seq(OptionalChain),
    },
);

pub const OPTIONAL_EXPR_MEMBER = 0;
pub const OPTIONAL_EXPR_CALL = 1;

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

pub const OPTIONAL_CHAIN_ARGS = 0;
pub const OPTIONAL_CHAIN_MEMBER = 1;
pub const OPTIONAL_CHAIN_PROPERTY = 2;
pub const OPTIONAL_CHAIN_PRIVATE_PROPERTY = 3;
pub const OPTIONAL_CHAIN_CHAIN_ARGS = 4;
pub const OPTIONAL_CHAIN_CHAIN_MEMBER = 5;
pub const OPTIONAL_CHAIN_CHAIN_PROPERTY = 6;
pub const OPTIONAL_CHAIN_CHAIN_PRIVATE_PROPERTY = 7;

pub const Arguments = extern struct {
    arguments: Seq(AssignmentExpression),
};

pub const SuperCall = Arguments;

pub const ImportCall = Seq(AssignmentExpression);

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

pub const CALL_EXPR_SUPER = 0;
pub const CALL_EXPR_IMPORT = 1;
pub const CALL_EXPR_SIMPLE_CALL = 2;
pub const CALL_EXPR_MEMBER = 3;
pub const CALL_EXPR_PROPERTY = 4;
pub const CALL_EXPR_PRIVATE_PROPERTY = 5;

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
        import_meta: MetaProperty,
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

pub const MEMBER_EXPR_PRIMARY = 0;
pub const MEMBER_EXPR_MEMBER = 1;
pub const MEMBER_EXPR_PROPERTY = 2;
pub const MEMBER_EXPR_SUPER = 3;
pub const MEMBER_EXPR_IMPORT_META = 4;
pub const MEMBER_EXPR_NEW = 5;
pub const MEMBER_EXPR_PRIVATE_PROPERTY = 6;

pub const SuperProperty = EXTERN_UNION(
    extern union {
        member: *Expression,
        property: *IdentifierNameData,
    },
);

pub const SUPER_PROP_MEMBER = 0;
pub const SUPER_PROP_PROPERTY = 1;

pub const MetaProperty = enum(u8) {
    NewTarget = 0,
    ImportMeta = 1,
};

pub const NewExpression = EXTERN_UNION(
    extern union {
        member: *MemberExpression,
        new: *NewExpression,
    },
);

pub const NEW_EXPR_MEMBER = 0;
pub const NEW_EXPR_NEW = 1;

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
            operator: AssignmentOperator,
            right: *AssignmentExpression,
        },
    },
);

pub const ASSIGNMENT_EXPR_CONDITIONAL = 0;
pub const ASSIGNMENT_EXPR_YIELD = 1;
pub const ASSIGNMENT_EXPR_RAW = 2;
pub const ASSIGNMENT_EXPR_OPERATOR = 3;

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
            condition: *ShortCircuitExpression,
            consequent: *AssignmentExpression,
            alternate: *AssignmentExpression,
        },
    },
);

pub const CONDITIONAL_EXPR_SHORT_CIRCUIT = 0;
pub const CONDITIONAL_EXPR_CONDITIONAL = 1;

pub const ShortCircuitExpression = EXTERN_UNION(
    extern union {
        logical_or: *LogicalORExpression,
        nullish_coalescing: *CoalescingExpression,
    },
);

pub const SHORT_CIRCUIT_EXPR_LOGICAL_OR = 0;
pub const SHORT_CIRCUIT_EXPR_NULLISH_COALESCING = 1;

pub const CoalescingExpression = extern struct {
    left: *CoalescingExpressionHead,
    right: *BitwiseORExpression,
};

pub const CoalescingExpressionHead = EXTERN_UNION(
    extern union {
        coalescing: *CoalescingExpression,
        bitwise_or: *BitwiseORExpression,
    },
);

pub const COALESCING_EXPR_HEAD_COALESCING = 0;
pub const COALESCING_EXPR_HEAD_BITWISE_OR = 1;

pub const LogicalORExpression = EXTERN_UNION(
    extern union {
        logical_and: *LogicalANDExpression,
        logical_or: extern struct {
            left: *LogicalORExpression,
            right: *LogicalANDExpression,
        },
    },
);

pub const LOGICAL_OR_EXPR_LOGICAL_AND = 0;
pub const LOGICAL_OR_EXPR_LOGICAL_OR = 1;

pub const LogicalANDExpression = EXTERN_UNION(
    extern union {
        bitwise_or: *BitwiseORExpression,
        logical_and: extern struct {
            left: *LogicalANDExpression,
            right: *BitwiseORExpression,
        },
    },
);

pub const LOGICAL_AND_EXPR_BITWISE_OR = 0;
pub const LOGICAL_AND_EXPR_LOGICAL_AND = 1;

pub const BitwiseORExpression = EXTERN_UNION(
    extern union {
        bitwise_xor: *BitwiseXORExpression,
        bitwise_or: extern struct {
            left: *BitwiseORExpression,
            right: *BitwiseXORExpression,
        },
    },
);

pub const BITWISE_OR_EXPR_BITWISE_XOR = 0;
pub const BITWISE_OR_EXPR_BITWISE_OR = 1;

pub const BitwiseXORExpression = EXTERN_UNION(
    extern union {
        bitwise_and: *BitwiseANDExpression,
        bitwise_xor: extern struct {
            left: *BitwiseXORExpression,
            right: *BitwiseANDExpression,
        },
    },
);

pub const BITWISE_XOR_EXPR_BITWISE_AND = 0;
pub const BITWISE_XOR_EXPR_BITWISE_XOR = 1;

pub const BitwiseANDExpression = EXTERN_UNION(
    extern union {
        equality: *EqualityExpression,
        bitwise_and: extern struct {
            left: *BitwiseANDExpression,
            right: *EqualityExpression,
        },
    },
);

pub const BITWISE_AND_EXPR_EQUALITY = 0;
pub const BITWISE_AND_EXPR_BITWISE_AND = 1;

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

pub const EQUALITY_EXPR_RELATIONAL = 0;
pub const EQUALITY_EXPR_EQUALITY = 1;

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

pub const RELATIONAL_EXPR_SHIFT = 0;
pub const RELATIONAL_EXPR_RELATIONAL = 1;
pub const RELATIONAL_EXPR_PRIVATE_IDENTIFIER_IN = 2;

pub const RelationalOperator = enum(u8) {
    LessThan = 0,
    GreaterThan = 1,
    LessThanOrEqual = 2,
    GreaterThanOrEqual = 3,
    InstanceOf = 4,
    In = 5,
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

pub const SHIFT_EXPR_ADDITIVE = 0;
pub const SHIFT_EXPR_SHIFT = 1;

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

pub const ADDITIVE_EXPR_MULTIPLICATIVE = 0;
pub const ADDITIVE_EXPR_ADDITIVE = 1;

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

pub const MULTIPLICATIVE_EXPR_EXPONENTIATION = 0;
pub const MULTIPLICATIVE_EXPR_MULTIPLICATIVE = 1;

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

pub const EXPONENTIATION_EXPR_UNARY = 0;
pub const EXPONENTIATION_EXPR_EXPONENTIATION = 1;

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

pub const UNARY_EXPR_UPDATE = 0;
pub const UNARY_EXPR_UNARY = 1;
pub const UNARY_EXPR_AWAIT = 2;

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

pub const UPDATE_EXPR_LEFT_HAND_SIDE = 0;
pub const UPDATE_EXPR_UPDATE = 1;

pub const UpdateOperator = enum(u8) {
    Increment = 0,
    Decrement = 1,
};

pub const IdentifierReference = EXTERN_UNION(
    extern union {
        identifier: *IdentifierNameData,
        yield: *void,
        await: *void,
    },
);

pub const IDENTIFIER_REF_IDENTIFIER = 0;
pub const IDENTIFIER_REF_YIELD = 1;
pub const IDENTIFIER_REF_AWAIT = 2;

pub const BindingIdentifier = EXTERN_UNION(
    extern union {
        identifier: *IdentifierNameData,
        yield: *void,
        await: *void,
    },
);

pub const BINDING_IDENTIFIER_IDENTIFIER = 0;
pub const BINDING_IDENTIFIER_YIELD = 1;
pub const BINDING_IDENTIFIER_AWAIT = 2;

pub const Literal = EXTERN_UNION(
    extern union {
        null: *void,
        boolean: bool,
        string: *String,
        number: *NumericLiteralData,
    },
);

pub const LITERAL_NULL = 0;
pub const LITERAL_BOOLEAN = 1;
pub const LITERAL_STRING = 2;
pub const LITERAL_NUMBER = 3;

pub const ArrayLiteral = extern struct {
    elements: Seq(ArrayElement),
};

pub const ArrayElement = EXTERN_UNION(
    extern union {
        expression: *AssignmentExpression,
        spread: *AssignmentExpression,
        ellision: void,
    },
);

pub const ARRAY_ELEMENT_EXPR = 0;
pub const ARRAY_ELEMENT_SPREAD = 1;
pub const ARRAY_ELEMENT_ELLISION = 2;

pub const ObjectLiteral = extern struct {
    properties: Seq(PropertyDefinition),
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

pub const PROPERTY_DEF_IDENTIFIER = 0;
pub const PROPERTY_DEF_COVER_INITIALIZED_NAME = 1;
pub const PROPERTY_DEF_PROPERTY = 2;
pub const PROPERTY_DEF_METHOD = 3;
pub const PROPERTY_DEF_SPREAD = 4;

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

pub const PROPERTY_NAME_IDENTIFIER = 0;
pub const PROPERTY_NAME_STRING_LITERAL = 1;
pub const PROPERTY_NAME_NUMERIC_LITERAL = 2;
pub const PROPERTY_NAME_COMPUTED = 3;

pub const Property = extern struct {
    key: *PropertyName,
    value: *AssignmentExpression,
};
