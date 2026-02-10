use std::fmt::Display;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ZigString {
    pub data: *const u16,
    pub len: usize,
}

impl Display for ZigString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.len) };
        let string = String::from_utf16_lossy(slice);
        write!(f, "{}", string)
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct IdentifierNameTokenData {
    pub name: ZigString,
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

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
    Whitespace = 0,
    LineTerminator = 1,
    Comment = 2,
    HashBangComment = 3,
    CommonToken = 4,
}

#[repr(C)]
#[derive(Debug)]
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

type CodePoint = u32;

#[link(name = "js", kind = "static")]
unsafe extern "C" {
    pub fn utf16_encode_cp(cp: CodePoint) -> ZigString;
    pub fn cps_to_string(cps: *const CodePoint, len: usize) -> ZigString;
    pub fn utf16_surrogate_pair_to_cp(high: u16, low: u16) -> CodePoint;
    pub fn code_point_at(s: ZigString, index: usize) -> CodePointAtResult;
    pub fn string_to_cps(text: ZigString) -> CodePointSeq;
    pub fn parse_text_string(text: ZigString, goal: u8) -> TokenSeq;
    pub fn parse_text_cps(text: CodePointSeq, goal: u8) -> TokenSeq;
    pub fn free_string(s: ZigString);
    pub fn free_code_point_seq(cps: CodePointSeq);
    pub fn free_token_seq(tokens: TokenSeq);
}
