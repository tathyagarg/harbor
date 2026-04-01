use std::fmt::{Debug, Display};

pub trait Seq {
    type Item;

    fn data(&self) -> *const Self::Item;
    fn len(&self) -> usize;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ZigString {
    pub data: *const u16,
    pub len: usize,
}

impl Seq for ZigString {
    type Item = u16;

    fn data(&self) -> *const Self::Item {
        self.data
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl Debug for ZigString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.len) };
        let string = String::from_utf16_lossy(slice);
        write!(f, "\"{}\"", string)?;
        write!(f, " [len={}]", self.len)?;

        Ok(())
    }
}

#[repr(C)]
pub struct CodePointAtResult {
    pub cp: CodePoint,
    pub code_unit_count: usize,
    pub is_unpaired_surrogate: bool,
}

#[repr(C)]
pub struct CodePointSeq {
    pub data: *const CodePoint,
    pub len: usize,
}

impl Seq for CodePointSeq {
    type Item = CodePoint;

    fn data(&self) -> *const Self::Item {
        self.data
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IdentifierNameTokenData {
    pub name: ZigString,
}

impl Debug for IdentifierNameTokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INTD({:?})", self.name)
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum NumericLiteralKind {
    Decimal,
    Binary,
    Octal,
    Hexadecimal,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NumericLiteralTokenData {
    pub value: f64,
    pub is_bigint: bool,
    pub number_kind: NumericLiteralKind,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct StringLiteralTokenData {
    pub value: ZigString,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CommonTokenData {
    pub common_token_kind: CommonTokenKind,
    pub value: usize,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CommonTokenKind {
    IdentifierName = 0,
    PrivateIdentifier = 1,
    Punctuator = 2,
    NumericLiteral = 3,
    StringLiteral = 4,
    Template = 5,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum PunctuatorKind {
    OptionalChaining,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Period,
    Ellipsis,
    Semicolon,
    Comma,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Equals,
    NotEquals,
    StrictEquals,
    StrictNotEquals,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Exponentiation,
    Increment,
    Decrement,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Not,
    BitwiseNot,
    LogicalAnd,
    LogicalOr,
    NullishCoalescing,
    QuestionMark,
    Colon,
    Assign,
    PlusAssign,
    MinusAssign,
    AsteriskAssign,
    SlashAssign,
    PercentAssign,
    ExponentiationAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishCoalescingAssign,
    FunctionArrow,
}

impl From<usize> for PunctuatorKind {
    fn from(value: usize) -> Self {
        unsafe { std::mem::transmute::<u8, PunctuatorKind>(value as u8) }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TokenKind {
    Whitespace = 0,
    LineTerminator = 1,
    Comment = 2,
    HashBangComment = 3,
    CommonToken = 4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct TokenSeq {
    pub data: *const Token,
    pub len: usize,
}

impl Display for CodePointSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.len) };
        for cp in slice {
            write!(f, "U+{:04X} ", cp)?;
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Expression {
    pub data: *const AssignmentExpression,
    pub len: usize,
}

impl Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.len) };
        for expr in slice {
            write!(f, "{:?} ", *expr)?;
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LeftHandSideExpression {
    pub tag: u8,
    pub data: LeftHandSideExpressionData,
}

impl Debug for LeftHandSideExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            LEFT_HAND_SIDE_EXPR_NEW => {
                let new_expr = unsafe { self.data.new };
                write!(f, "NewExpression({:?})", unsafe { *new_expr })
            }
            LEFT_HAND_SIDE_EXPR_CALL => {
                let call_expr = unsafe { self.data.call };
                write!(f, "CallExpression({:?})", unsafe { *call_expr })
            }
            LEFT_HAND_SIDE_EXPR_OPTIONAL => {
                let optional_expr = unsafe { self.data.optional };
                write!(f, "OptionalExpression({:?})", unsafe { *optional_expr })
            }
            _ => write!(f, "UnknownLeftHandSideExpression"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LeftHandSideExpressionData {
    pub new: *const NewExpression,
    pub call: *const CallExpression,
    pub optional: *const OptionalExpression,
}

pub const LEFT_HAND_SIDE_EXPR_NEW: u8 = 0;
pub const LEFT_HAND_SIDE_EXPR_CALL: u8 = 1;
pub const LEFT_HAND_SIDE_EXPR_OPTIONAL: u8 = 2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OptionalExpression {
    pub tag: u8,
    pub data: OptionalExpressionData,
}

impl Debug for OptionalExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            OPTIONAL_EXPR_MEMBER => {
                let member_expr = unsafe { self.data.kind.member };
                write!(f, "OptionalMemberExpression({:?})", unsafe { *member_expr })
            }
            OPTIONAL_EXPR_CALL => {
                let call_expr = unsafe { self.data.kind.call };
                write!(f, "OptionalCallExpression({:?})", unsafe { *call_expr })
            }
            _ => write!(f, "UnknownOptionalExpression"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OptionalExpressionData {
    pub kind: OptionalExpressionKind,
    pub chains: OptionalChainSeq,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union OptionalExpressionKind {
    member: *const MemberExpression,
    call: *const CallExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OptionalChainSeq {
    pub data: *const OptionalChain,
    pub len: usize,
}

pub const OPTIONAL_EXPR_MEMBER: u8 = 0;
pub const OPTIONAL_EXPR_CALL: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OptionalChain {
    pub tag: u8,
    pub data: OptionalChainData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union OptionalChainData {
    args: *const Arguments,
    member: *const Expression,
    property: *const IdentifierNameTokenData,
    private_property: *const IdentifierNameTokenData,
    chain_args: ChainArgsData,
    chain_member: ChainMemberData,
    chain_property: ChainPropertyData,
    chain_private_property: ChainPrivatePropertyData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChainArgsData {
    pub chain: *const OptionalChain,
    pub arguments: *const Arguments,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChainMemberData {
    pub chain: *const OptionalChain,
    pub expr: *const Expression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChainPropertyData {
    pub chain: *const OptionalChain,
    pub property: *const IdentifierNameTokenData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChainPrivatePropertyData {
    pub chain: *const OptionalChain,
    pub property: *const IdentifierNameTokenData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NewExpression {
    pub tag: u8,
    pub data: NewExpressionData,
}

impl Debug for NewExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            NEW_EXPR_MEMBER => {
                let member_expr = unsafe { self.data.member };
                write!(f, "NewMemberExpression({:?})", unsafe { *member_expr })
            }
            NEW_EXPR_NEW => {
                let new_expr = unsafe { self.data.new };
                write!(f, "NewExpression({:?})", unsafe { *new_expr })
            }
            _ => write!(f, "UnknownNewExpression"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union NewExpressionData {
    pub member: *const MemberExpression,
    pub new: *const NewExpression,
}

pub const NEW_EXPR_MEMBER: u8 = 0;
pub const NEW_EXPR_NEW: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemberExpression {
    pub tag: u8,
    pub data: MemberExpressionData,
}

impl Debug for MemberExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            MEMBER_EXPR_PRIMARY => {
                let primary_expr = unsafe { self.data.primary };
                write!(f, "PrimaryExpression({:?})", unsafe { *primary_expr })
            }
            MEMBER_EXPR_MEMBER => {
                let member_data = unsafe { self.data.member };
                write!(
                    f,
                    "MemberExpression(object={:?}, expr={:?})",
                    unsafe { *member_data.object },
                    unsafe { *member_data.expr }
                )
            }
            MEMBER_EXPR_PROPERTY => {
                let property_data = unsafe { self.data.property };
                write!(
                    f,
                    "MemberExpression(object={:?}, property={:?})",
                    unsafe { *property_data.object },
                    unsafe { *property_data.property }
                )
            }
            MEMBER_EXPR_SUPER => {
                let super_data = unsafe { self.data._super };
                write!(f, "SuperProperty({:?})", unsafe { *super_data })
            }
            MEMBER_EXPR_IMPORT_META => {
                let import_meta = unsafe { self.data.import_meta };
                write!(f, "ImportMeta({:?})", import_meta)
            }
            MEMBER_EXPR_NEW => {
                let new_member_data = unsafe { self.data.new };
                write!(f, "NewMemberExpression({:?})", new_member_data)
            }
            MEMBER_EXPR_PRIVATE_PROPERTY => {
                let private_member_data = unsafe { self.data.private_property };
                write!(
                    f,
                    "PrivateMemberExpression(object={:?}, property={:?})",
                    unsafe { *private_member_data.object },
                    unsafe { *private_member_data.property }
                )
            }
            _ => write!(f, "UnknownMemberExpression"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PrimaryExpression {
    pub tag: u8,
    pub data: PrimaryExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union PrimaryExpressionData {
    pub this: (),
    pub identifier: *const IdentifierReference,
    pub literal: *const Literal,
    pub array: *const ArrayLiteral,
    pub object: *const ObjectLiteral,
}

pub const PRIMARY_EXPR_THIS: u8 = 0;
pub const PRIMARY_EXPR_IDENTIFIER: u8 = 1;
pub const PRIMARY_EXPR_LITERAL: u8 = 2;
pub const PRIMARY_EXPR_ARRAY: u8 = 3;
pub const PRIMARY_EXPR_OBJECT: u8 = 4;

impl Debug for PrimaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            PRIMARY_EXPR_THIS => write!(f, "ThisExpression"),
            PRIMARY_EXPR_IDENTIFIER => {
                let identifier_ref = unsafe { self.data.identifier };
                write!(f, "IdentifierReference({:?})", unsafe { *identifier_ref })
            }
            PRIMARY_EXPR_LITERAL => {
                let literal = unsafe { self.data.literal };
                write!(f, "Literal({:?})", unsafe { *literal })
            }
            PRIMARY_EXPR_ARRAY => {
                let array = unsafe { self.data.array };
                write!(f, "ArrayLiteral[\n")?;

                let elements = unsafe {
                    std::slice::from_raw_parts((*array).elements.data, (*array).elements.len)
                };

                for element in elements {
                    match element.tag {
                        ARRAY_ELEMENT_EXPRESSION => {
                            let expr = unsafe { element.data.expression };
                            write!(f, "    Expression({:?}),\n", unsafe { *expr })?;
                        }
                        ARRAY_ELEMENT_SPREAD => {
                            let spread = unsafe { element.data.spread };
                            write!(f, "    Spread({:?}),\n", unsafe { *spread })?;
                        }
                        ARRAY_ELEMENT_ELISION => {
                            write!(f, "    Elision,\n")?;
                        }
                        _ => panic!("Unknown ArrayElement tag: {}", element.tag),
                    }
                }

                write!(f, "]")?;

                Ok(())
            }
            PRIMARY_EXPR_OBJECT => {
                let object = unsafe { self.data.object };
                write!(f, "ObjectLiteral({:p})", object)
            }
            _ => panic!("Unknown PrimaryExpression tag: {}", self.tag),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IdentifierReference {
    pub tag: u8,
    pub data: IdentifierReferenceData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union IdentifierReferenceData {
    identifier: *const IdentifierNameTokenData,
    _yield: (),
    _await: (),
}

pub const IDENTIFIER_REF_IDENTIFIER: u8 = 0;
pub const IDENTIFIER_REF_YIELD: u8 = 1;
pub const IDENTIFIER_REF_AWAIT: u8 = 2;

impl Debug for IdentifierReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            IDENTIFIER_REF_IDENTIFIER => {
                let identifier_name = unsafe { self.data.identifier };
                write!(f, "Identifier({:?})", unsafe { *identifier_name })
            }
            IDENTIFIER_REF_YIELD => write!(f, "Yield"),
            IDENTIFIER_REF_AWAIT => write!(f, "Await"),
            _ => write!(f, "UnknownIdentifierReference"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Literal {
    pub tag: u8,
    pub data: LiteralData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LiteralData {
    pub null: (),
    pub boolean: bool,
    pub string: *const ZigString,
    pub numeric: *const NumericLiteralTokenData,
}

pub const LITERAL_NULL: u8 = 0;
pub const LITERAL_BOOLEAN: u8 = 1;
pub const LITERAL_STRING: u8 = 2;
pub const LITERAL_NUMBER: u8 = 3;

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            LITERAL_NULL => write!(f, "NullLiteral"),
            LITERAL_BOOLEAN => {
                let value = unsafe { self.data.boolean };
                write!(f, "BooleanLiteral({})", value)
            }
            LITERAL_STRING => {
                let string = unsafe { self.data.string };
                write!(f, "StringLiteral({:?})", unsafe { *string })
            }
            LITERAL_NUMBER => {
                let numeric_data = unsafe { self.data.numeric };
                write!(
                    f,
                    "NumericLiteral(value={}, is_bigint={}, kind={:?})",
                    unsafe { (*numeric_data).value },
                    unsafe { (*numeric_data).is_bigint },
                    unsafe { (*numeric_data).number_kind }
                )
            }
            _ => write!(f, "UnknownLiteral"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ArrayLiteral {
    pub elements: ArrayElementSeq,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ArrayElementSeq {
    pub data: *const ArrayElement,
    pub len: usize,
}

impl Seq for ArrayElementSeq {
    type Item = ArrayElement;

    fn data(&self) -> *const Self::Item {
        self.data
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ArrayElement {
    pub tag: u8,
    pub data: ArrayElementData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ArrayElementData {
    expression: *const AssignmentExpression,
    spread: *const AssignmentExpression,
    elision: (),
}

pub const ARRAY_ELEMENT_EXPRESSION: u8 = 0;
pub const ARRAY_ELEMENT_SPREAD: u8 = 1;
pub const ARRAY_ELEMENT_ELISION: u8 = 2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ObjectLiteral {
    pub properties: ObjectPropertySeq,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ObjectPropertySeq {
    pub data: *const ObjectProperty,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ObjectProperty {
    pub tag: u8,
    pub data: ObjectPropertyData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ObjectPropertyData {
    identifier: *const IdentifierReference,
    cover_initialized: *const CoverInitializedName,
    property: *const PropertyDefinition,
    method: (), // TODO:
    spread: *const AssignmentExpression,
}

pub const OBJECT_PROPERTY_IDENTIFIER: u8 = 0;
pub const OBJECT_PROPERTY_COVER_INITIALIZED: u8 = 1;
pub const OBJECT_PROPERTY_PROPERTY: u8 = 2;
pub const OBJECT_PROPERTY_METHOD: u8 = 3;
pub const OBJECT_PROPERTY_SPREAD: u8 = 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoverInitializedName {
    pub identifier: *const IdentifierNameTokenData,
    pub initializer: *const AssignmentExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PropertyDefinition {
    pub key: *const PropertyName,
    pub value: *const AssignmentExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PropertyName {
    pub tag: u8,
    pub data: PropertyNameData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union PropertyNameData {
    identifier_name: *const IdentifierNameTokenData,
    string: *const ZigString,
    numeric: *const NumericLiteralTokenData,
    computed: *const AssignmentExpression,
}

pub const PROPERTY_NAME_IDENTIFIER: u8 = 0;
pub const PROPERTY_NAME_STRING: u8 = 1;
pub const PROPERTY_NAME_NUMERIC: u8 = 2;
pub const PROPERTY_NAME_COMPUTED: u8 = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub union MemberExpressionData {
    pub primary: *const PrimaryExpression,
    pub member: TrueMemberExpressionData,
    pub property: PropertyMemberExpressionData,
    pub _super: *const SuperProperty,
    pub import_meta: ImportMeta,
    pub new: NewMemberExpressionData,
    pub private_property: PrivateMemberExpressionData,
}

pub const MEMBER_EXPR_PRIMARY: u8 = 0;
pub const MEMBER_EXPR_MEMBER: u8 = 1;
pub const MEMBER_EXPR_PROPERTY: u8 = 2;
pub const MEMBER_EXPR_SUPER: u8 = 3;
pub const MEMBER_EXPR_IMPORT_META: u8 = 4;
pub const MEMBER_EXPR_NEW: u8 = 5;
pub const MEMBER_EXPR_PRIVATE_PROPERTY: u8 = 6;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueMemberExpressionData {
    pub object: *const MemberExpression,
    pub expr: *const Expression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PropertyMemberExpressionData {
    pub object: *const MemberExpression,
    pub property: *const IdentifierNameTokenData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SuperProperty {
    pub tag: u8,
    pub data: SuperPropertyData,
}

impl Debug for SuperProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            SUPER_PROPERTY_MEMBER => {
                let member = unsafe { self.data.member };
                write!(f, "SuperMemberExpression({:?})", unsafe { *member })
            }
            SUPER_PROPERTY_PROPERTY => {
                let property = unsafe { self.data.property };
                write!(f, "SuperProperty({:?})", unsafe { *property })
            }
            _ => write!(f, "UnknownSuperProperty"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union SuperPropertyData {
    member: *const Expression,
    property: *const IdentifierNameTokenData,
}

pub const SUPER_PROPERTY_MEMBER: u8 = 0;
pub const SUPER_PROPERTY_PROPERTY: u8 = 1;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ImportMeta {
    NewTarget = 0,
    ImportMeta = 1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NewMemberExpressionData {
    pub callee: *const MemberExpression,
    pub arguments: *const Arguments,
}

impl Debug for NewMemberExpressionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NewMemberExpression(callee={:?}, arguments={:?})",
            unsafe { *self.callee },
            unsafe { *self.arguments }
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Arguments {
    pub arguments: AssignmentExpressionSeq,
    pub is_spread: *const bool,
}

impl Debug for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = unsafe { std::slice::from_raw_parts(self.arguments.data, self.arguments.len) };
        write!(f, "Arguments(")?;
        for arg in args {
            write!(f, "{:?} ", *arg)?;
        }
        write!(f, ")")
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AssignmentExpressionSeq {
    pub data: *const AssignmentExpression,
    pub len: usize,
}

impl Seq for AssignmentExpressionSeq {
    type Item = AssignmentExpression;

    fn data(&self) -> *const Self::Item {
        self.data
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PrivateMemberExpressionData {
    pub object: *const MemberExpression,
    pub property: *const IdentifierNameTokenData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CallExpression {
    pub tag: u8,
    pub data: CallExpressionData,
}

impl Debug for CallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            CALL_EXPR_SUPER => {
                let super_call = unsafe { self.data._super };
                write!(f, "SuperCall({:?})", unsafe { *super_call })
            }
            CALL_EXPR_IMPORT => {
                let import_call = unsafe { self.data.import };
                write!(
                    f,
                    "ImportCall(data={:?}, len={})",
                    unsafe { (*import_call).data },
                    unsafe { (*import_call).len }
                )
            }
            CALL_EXPR_CALL => {
                let call_data = unsafe { self.data.call };
                write!(
                    f,
                    "CallExpression(callee={:?}, arguments={:?})",
                    unsafe { *call_data.callee },
                    unsafe { *call_data.arguments }
                )
            }
            CALL_EXPR_MEMBER => {
                let member_call_data = unsafe { self.data.member };
                write!(
                    f,
                    "MemberCallExpression(object={:?}, expr={:?})",
                    unsafe { *member_call_data.object },
                    unsafe { *member_call_data.expr }
                )
            }
            CALL_EXPR_PROPERTY => {
                let property_call_data = unsafe { self.data.property };
                write!(
                    f,
                    "PropertyCallExpression(object={:?}, property={:?})",
                    unsafe { *property_call_data.object },
                    unsafe { *property_call_data.property }
                )
            }
            CALL_EXPR_PRIVATE_PROPERTY => {
                let private_property_call_data = unsafe { self.data.private_property };
                write!(
                    f,
                    "PrivatePropertyCallExpression(object={:?}, property={:?})",
                    unsafe { *private_property_call_data.object },
                    unsafe { *private_property_call_data.property }
                )
            }
            CALL_EXPR_COVER => {
                let cover_call = unsafe { self.data.cover };
                write!(
                    f,
                    "CoverCallExpression(callee={:?}, arguments={:?})",
                    unsafe { *cover_call.callee },
                    unsafe { *cover_call.arguments }
                )
            }
            _ => write!(f, "UnknownCallExpression"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoverCallExpression {
    pub callee: *const MemberExpression,
    pub arguments: *const Arguments,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union CallExpressionData {
    pub _super: *const SuperCall,
    pub import: *const ImportCall,
    pub call: TrueCallExpressionData,
    pub member: MemberCallExpressionData,
    pub property: PropertyCallExpressionData,
    pub private_property: PrivateCallExpressionData,
    pub cover: CoverCallExpression,
}

pub const CALL_EXPR_SUPER: u8 = 0;
pub const CALL_EXPR_IMPORT: u8 = 1;
pub const CALL_EXPR_CALL: u8 = 2;
pub const CALL_EXPR_MEMBER: u8 = 3;
pub const CALL_EXPR_PROPERTY: u8 = 4;
pub const CALL_EXPR_PRIVATE_PROPERTY: u8 = 5;
pub const CALL_EXPR_COVER: u8 = 6;

pub type SuperCall = Arguments;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImportCall {
    pub data: *const AssignmentExpression,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueCallExpressionData {
    pub callee: *const CallExpression,
    pub arguments: *const Arguments,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemberCallExpressionData {
    pub object: *const CallExpression,
    pub expr: *const Expression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PropertyCallExpressionData {
    pub object: *const CallExpression,
    pub property: *const IdentifierNameTokenData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PrivateCallExpressionData {
    pub object: *const CallExpression,
    pub property: *const IdentifierNameTokenData,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RawAssignmentData {
    pub left: *const LeftHandSideExpression,
    pub right: *const AssignmentExpression,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OperatorAssignmentExpressionData {
    pub left: *const LeftHandSideExpression,
    pub operator: AssignmentOperator,
    pub right: *const AssignmentExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AssignmentOperator {
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

    Raw = 15,
}

#[repr(C)]
pub struct TernaryExpression {
    pub test: *const AssignmentExpression,
    pub consequent: *const AssignmentExpression,
    pub alternate: *const AssignmentExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BinaryOrUnaryExpression {
    pub tag: u8,
    pub data: BinaryOrUnaryExpressionData,
}

impl Debug for BinaryOrUnaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            BINARY_OR_UNARY_EXPR_BINARY => {
                let binary_expr = unsafe { self.data.binary };
                write!(f, "BinaryExpression({:?})", unsafe { *binary_expr })
            }
            BINARY_OR_UNARY_EXPR_UNARY => {
                let unary_expr = unsafe { self.data.unary };
                write!(f, "UnaryExpression({:?})", unsafe { *unary_expr })
            }
            BINARY_OR_UNARY_EXPR_NONE => write!(f, "None"),
            _ => write!(f, "UnknownBinaryOrUnaryExpression"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union BinaryOrUnaryExpressionData {
    binary: *const BinaryExpression,
    unary: *const UnaryExpression,
    none: (),
}

pub const BINARY_OR_UNARY_EXPR_BINARY: u8 = 0;
pub const BINARY_OR_UNARY_EXPR_UNARY: u8 = 1;
pub const BINARY_OR_UNARY_EXPR_NONE: u8 = 2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnaryExpressionOrNull {
    pub tag: u8,
    pub data: UnaryExpressionOrNullData,
}

impl Debug for UnaryExpressionOrNull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            UNARY_EXPR_OR_NULL_UNARY => {
                let unary_expr = unsafe { self.data.unary };
                write!(f, "UnaryExpression({:?})", unsafe { *unary_expr })
            }
            UNARY_EXPR_OR_NULL_NONE => write!(f, "None"),
            _ => write!(f, "UnknownUnaryExpressionOrNull"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union UnaryExpressionOrNullData {
    unary: *const UnaryExpression,
    none: (),
}

pub const UNARY_EXPR_OR_NULL_UNARY: u8 = 0;
pub const UNARY_EXPR_OR_NULL_NONE: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BinaryExpression {
    pub left: *const BinaryOrUnaryExpression,
    pub operator: BinaryOperator,
    pub right: *const UnaryExpressionOrNull,
}

impl Debug for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BinaryExpression(left={:?}, operator={:?}, right={:?})",
            unsafe { *self.left },
            self.operator,
            unsafe { *self.right }
        )
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum BinaryOperator {
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

    None = 22,

    BitwiseAnd = 23,
    BitwiseXor = 24,
    BitwiseOr = 25,
    LogicalAnd = 26,
    LogicalOr = 27,
}

#[repr(C)]
pub struct ShortCircuitExpression {
    pub tag: u8,
    pub data: ShortCircuitExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ShortCircuitExpressionData {
    logical_or: *const LogicalOrExpression,
    nullish_coalescing: *const NullishCoalescingExpression,
}

pub const SHORT_CIRCUIT_EXPR_LOGICAL_OR: u8 = 0;
pub const SHORT_CIRCUIT_EXPR_NULLISH_COALESCING: u8 = 1;

#[repr(C)]
pub struct NullishCoalescingExpression {
    pub left: *const CoalescingExpressionHead,
    pub right: *const BitwiseOrExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoalescingExpressionHead {
    pub tag: u8,
    pub data: CoalescingExpressionHeadData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union CoalescingExpressionHeadData {
    nullish_coalescing: *const NullishCoalescingExpression,
    bitwise_or: *const BitwiseOrExpression,
}

pub const COALESCING_EXPR_HEAD_NULLISH_COALESCING: u8 = 0;
pub const COALESCING_EXPR_HEAD_BITWISE_OR: u8 = 1;

#[repr(C)]
pub struct LogicalOrExpression {
    pub tag: u8,
    pub data: LogicalOrExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LogicalOrExpressionData {
    logical_and: *const LogicalAndExpression,
    logical_or: TrueLogicalOrExpressionData,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrueLogicalOrExpressionData {
    pub left: *const LogicalOrExpression,
    pub right: *const LogicalAndExpression,
}

pub const LOGICAL_OR_EXPR_LOGICAL_AND: u8 = 0;
pub const LOGICAL_OR_EXPR_LOGICAL_OR: u8 = 1;

#[repr(C)]
pub struct LogicalAndExpression {
    pub tag: u8,
    pub data: LogicalAndExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LogicalAndExpressionData {
    bitwise_or: *const BitwiseOrExpression,
    logical_and: TrueLogicalAndExpressionData,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrueLogicalAndExpressionData {
    pub left: *const LogicalAndExpression,
    pub right: *const BitwiseOrExpression,
}

pub const LOGICAL_AND_EXPR_BITWISE_OR: u8 = 0;
pub const LOGICAL_AND_EXPR_LOGICAL_AND: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BitwiseOrExpression {
    pub tag: u8,
    pub data: BitwiseOrExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union BitwiseOrExpressionData {
    bitwise_xor: *const BitwiseXorExpression,
    bitwise_or: TrueBitwiseOrExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueBitwiseOrExpressionData {
    pub left: *const BitwiseOrExpression,
    pub right: *const BitwiseXorExpression,
}

pub const BITWISE_OR_EXPR_BITWISE_XOR: u8 = 0;
pub const BITWISE_OR_EXPR_BITWISE_OR: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BitwiseXorExpression {
    pub tag: u8,
    pub data: BitwiseXorExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union BitwiseXorExpressionData {
    bitwise_and: *const BitwiseAndExpression,
    bitwise_xor: TrueBitwiseXorExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueBitwiseXorExpressionData {
    pub left: *const BitwiseXorExpression,
    pub right: *const BitwiseAndExpression,
}

pub const BITWISE_XOR_EXPR_BITWISE_AND: u8 = 0;
pub const BITWISE_XOR_EXPR_BITWISE_XOR: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BitwiseAndExpression {
    pub tag: u8,
    pub data: BitwiseAndExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union BitwiseAndExpressionData {
    equality: *const EqualityExpression,
    bitwise_and: TrueBitwiseAndExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueBitwiseAndExpressionData {
    pub left: *const BitwiseAndExpression,
    pub right: *const EqualityExpression,
}

pub const BITWISE_AND_EXPR_EQUALITY: u8 = 0;
pub const BITWISE_AND_EXPR_BITWISE_AND: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EqualityExpression {
    pub tag: u8,
    pub data: EqualityExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union EqualityExpressionData {
    relational: *const RelationalExpression,
    equality: TrueEqualityExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueEqualityExpressionData {
    pub left: *const EqualityExpression,
    pub operator: EqualityOperator,
    pub right: *const RelationalExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum EqualityOperator {
    Equal = 0,
    NotEqual = 1,
    StrictEqual = 2,
    StrictNotEqual = 3,
}

pub const EQUALITY_EXPR_RELATIONAL: u8 = 0;
pub const EQUALITY_EXPR_EQUALITY: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RelationalExpression {
    pub tag: u8,
    pub data: RelationalExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RelationalExpressionData {
    shift: *const ShiftExpression,
    relational: TrueRelationalExpressionData,
    private_identifier_in: PrivateIdentifierInData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueRelationalExpressionData {
    pub left: *const RelationalExpression,
    pub operator: RelationalOperator,
    pub right: *const ShiftExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RelationalOperator {
    LessThan = 0,
    GreaterThan = 1,
    LessThanEqual = 2,
    GreaterThanEqual = 3,
    InstanceOf = 4,
    In = 5,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PrivateIdentifierInData {
    pub left: *const IdentifierNameTokenData,
    pub right: *const ShiftExpression,
}

pub const RELATIONAL_EXPR_SHIFT: u8 = 0;
pub const RELATIONAL_EXPR_RELATIONAL: u8 = 1;
pub const RELATIONAL_EXPR_PRIVATE_IDENTIFIER_IN: u8 = 2;

#[repr(C)]
pub struct ShiftExpression {
    pub tag: u8,
    pub data: ShiftExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ShiftExpressionData {
    additive: *const AdditiveExpression,
    shift: TrueShiftExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueShiftExpressionData {
    pub left: *const ShiftExpression,
    pub operator: ShiftOperator,
    pub right: *const AdditiveExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShiftOperator {
    LeftShift = 0,
    RightShift = 1,
    UnsignedRightShift = 2,
}

pub const SHIFT_EXPR_ADDITIVE: u8 = 0;
pub const SHIFT_EXPR_SHIFT: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AdditiveExpression {
    pub tag: u8,
    pub data: AdditiveExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union AdditiveExpressionData {
    multiplicative: *const MultiplicativeExpression,
    additive: TrueAdditiveExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueAdditiveExpressionData {
    pub left: *const AdditiveExpression,
    pub operator: AdditiveOperator,
    pub right: *const MultiplicativeExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AdditiveOperator {
    Plus = 0,
    Minus = 1,
}

pub const ADDITIVE_EXPR_MULTIPLICATIVE: u8 = 0;
pub const ADDITIVE_EXPR_ADDITIVE: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MultiplicativeExpression {
    pub tag: u8,
    pub data: MultiplicativeExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union MultiplicativeExpressionData {
    exponentiation: *const ExponentiationExpression,
    multiplicative: TrueMultiplicativeExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueMultiplicativeExpressionData {
    pub left: *const MultiplicativeExpression,
    pub operator: MultiplicativeOperator,
    pub right: *const ExponentiationExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum MultiplicativeOperator {
    Star = 0,
    Slash = 1,
    Percent = 2,
}

pub const MULTIPLICATIVE_EXPR_EXPONENTIATION: u8 = 0;
pub const MULTIPLICATIVE_EXPR_MULTIPLICATIVE: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ExponentiationExpression {
    pub tag: u8,
    pub data: ExponentiationExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ExponentiationExpressionData {
    unary: *const UnaryExpression,
    exponentiation: TrueExponentiationExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrueExponentiationExpressionData {
    pub left: *const UnaryExpression,
    pub right: *const ExponentiationExpression,
}

pub const EXPONENTIATION_EXPR_UNARY: u8 = 0;
pub const EXPONENTIATION_EXPR_EXPONENTIATION: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: *const UnaryExpressionOrLHS,
}

impl Debug for UnaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UnaryExpression(operator={:?}, operand={:?})",
            self.operator,
            unsafe { *self.operand },
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnaryExpressionOrLHS {
    pub tag: u8,
    pub data: UnaryExpressionOrLHSData,
}

const UNARY_EXPR_OR_LHS_UNARY: u8 = 0;
const UNARY_EXPR_OR_LHS_LHS: u8 = 1;

impl Debug for UnaryExpressionOrLHS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            UNARY_EXPR_OR_LHS_UNARY => {
                let unary_expr = unsafe { *self.data.unary };
                write!(f, "UnaryExpression({:?})", unary_expr)
            }
            UNARY_EXPR_OR_LHS_LHS => {
                let lhs_expr = unsafe { self.data.lhs };
                write!(f, "LeftHandSideExpression({:?})", unsafe { *lhs_expr })
            }
            _ => write!(f, "UnknownUnaryExpressionOrLHS"),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union UnaryExpressionOrLHSData {
    pub unary: *const UnaryExpression,
    pub lhs: *const LeftHandSideExpression,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum UnaryOperator {
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

    None = 12,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AwaitExpression {
    pub expr: *const UnaryExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct YieldExpression {
    pub tag: u8,
    pub data: YieldExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union YieldExpressionData {
    none: (),
    expr: *const AssignmentExpression,
    delegation: *const AssignmentExpression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AssignmentExpression {
    pub tag: u8,
    pub data: AssignmentExpressionData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union AssignmentExpressionData {
    _yield: *const YieldExpression,
    raw_assignment: RawAssignmentData,
    operator_assignment: OperatorAssignmentExpressionData,
    ternary: *const TernaryExpression,
    binary: *const BinaryExpression,
    unary: *const UnaryExpression,
    primary: *const PrimaryExpression,
    lhs: *const LeftHandSideExpression,
}

pub const ASSIGNMENT_EXPR_CONDITIONAL: u8 = 0;
pub const ASSIGNMENT_EXPR_YIELD: u8 = 1;
pub const ASSIGNMENT_EXPR_RAW: u8 = 2;
pub const ASSIGNMENT_EXPR_OPERATOR: u8 = 3;
pub const ASSIGNMENT_EXPR_TERNARY: u8 = 4;
pub const ASSIGNMENT_EXPR_BINARY: u8 = 5;
pub const ASSIGNMENT_EXPR_UNARY: u8 = 6;
pub const ASSIGNMENT_EXPR_PRIMARY: u8 = 7;
pub const ASSIGNMENT_EXPR_LHS: u8 = 8;

impl Debug for AssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            ASSIGNMENT_EXPR_YIELD => {
                write!(f, "YieldExpression({:p})", unsafe { self.data._yield })
            }
            ASSIGNMENT_EXPR_RAW => {
                let raw = unsafe { self.data.raw_assignment };
                write!(
                    f,
                    "RawAssignment(left: {:?}, right: {:?})",
                    unsafe { *raw.left },
                    unsafe { *raw.right }
                )
            }
            ASSIGNMENT_EXPR_OPERATOR => {
                let op = unsafe { self.data.operator_assignment };
                write!(
                    f,
                    "OperatorAssignment(left: {:p}, operator: {:?}, right: {:p})",
                    op.left, op.operator, op.right
                )
            }
            ASSIGNMENT_EXPR_TERNARY => {
                write!(
                    f,
                    "TernaryExpression(test: {:p}, consequent: {:p}, alternate: {:p})",
                    unsafe { (*self.data.ternary).test },
                    unsafe { (*self.data.ternary).consequent },
                    unsafe { (*self.data.ternary).alternate },
                )
            }
            ASSIGNMENT_EXPR_BINARY => {
                let binary = unsafe { self.data.binary };
                write!(
                    f,
                    "BinaryExpression(left: {:?}, operator: {:?}, right: {:?})",
                    unsafe { *(*binary).left },
                    unsafe { (*binary).operator },
                    unsafe { *(*binary).right },
                )
            }
            ASSIGNMENT_EXPR_UNARY => {
                let unary = unsafe { self.data.unary };
                write!(
                    f,
                    "UnaryExpression(operator: {:?}, operand: {:p})",
                    unsafe { (*unary).operator },
                    unsafe { (*unary).operand },
                )
            }
            ASSIGNMENT_EXPR_PRIMARY => {
                write!(f, "PrimaryExpression({:?})", unsafe { *self.data.primary })
            }
            ASSIGNMENT_EXPR_LHS => {
                write!(f, "LeftHandSideExpression({:?})", unsafe { *self.data.lhs })
            }
            _ => write!(f, "UnknownAssignmentExpression(tag: {})", self.tag),
        }
    }
}

pub type CodePoint = u32;
