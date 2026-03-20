pub mod expr;
pub mod stmt;

use expr::{CodePoint, CodePointAtResult, CodePointSeq, TokenSeq, ZigString};

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

    // pub fn temp_unsafe_parse_primary_expr(tokens: TokenSeq) -> PrimaryExpression;
    pub fn parse_script(text: ZigString) -> stmt::Script;
}
