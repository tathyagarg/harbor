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
        this: void,
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
        ternary: *TernaryExpression,
        binary: *BinaryExpression,
        unary: *UnaryExpression,
        primary: *PrimaryExpression,
    },
);

// 0: deprecated
pub const ASSIGNMENT_EXPR_CONDITIONAL = 0;
pub const ASSIGNMENT_EXPR_YIELD = 1;
pub const ASSIGNMENT_EXPR_RAW = 2;
pub const ASSIGNMENT_EXPR_OPERATOR = 3;
pub const ASSIGNMENT_EXPR_TERNARY = 4;
pub const ASSIGNMENT_EXPR_BINARY = 5;
pub const ASSIGNMENT_EXPR_UNARY = 6;
pub const ASSIGNMENT_EXPR_PRIMARY = 7;

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

// deprecated
pub const CONDITIONAL_EXPR_SHORT_CIRCUIT = 0;
pub const CONDITIONAL_EXPR_CONDITIONAL = 1;

pub const TernaryExpression = extern struct {
    condition: *AssignmentExpression,
    consequent: *AssignmentExpression,
    alternate: *AssignmentExpression,
};

pub const BinaryOrUnaryExpression = EXTERN_UNION(
    extern union {
        binary: *BinaryExpression,
        unary: *UnaryExpression,
    },
);

pub const BinaryExpression = extern struct {
    left: *AssignmentExpression,
    operator: BinaryOperator,
    right: *AssignmentExpression,
};

pub const BinaryOperator = enum(u8) {
    Equal = 0,
    NotEqual = 1,
    StrictEqual = 2,
    StrictNotEqual = 3,
    LessThan = 4,
    GreaterThan = 5,
    LessThanOrEqual = 6,
    GreaterThanOrEqual = 7,
    InstanceOf = 8,
    In = 9,
    LeftShift = 10,
    RightShift = 11,
    UnsignedRightShift = 12,
    Plus = 13,
    Minus = 14,
    Star = 15,
    Slash = 16,
    Percent = 17,
    Exponentiation = 18,
    ShortCircuitLogicalAnd = 19,
    ShortCircuitLogicalOr = 20,
    NullishCoalescing = 21,
};

pub const UnaryExpression = extern struct {
    operator: *UnaryOperator,
    operand: *UnaryExpressionOrLHS,
};

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
    PrefixIncrement = 7,
    PrefixDecrement = 8,
    PostfixIncrement = 9,
    PostfixDecrement = 10,
    Await = 11,
};

pub const UnaryExpressionOrLHS = EXTERN_UNION(
    extern union {
        unary: *UnaryExpression,
        left_hand_side: *LeftHandSideExpression,
    },
);

pub const UNARY_EXPR_OR_LHS_UNARY = 0;
pub const UNARY_EXPR_OR_LHS_LHS = 1;

pub const AwaitExpression = extern struct {
    expression: *UnaryExpression,
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
