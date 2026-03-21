pub mod executable;
pub mod expr;
pub mod operations;
pub mod semantics;
pub mod stmt;
pub mod types;
pub mod values;

use expr::{CodePoint, CodePointAtResult, CodePointSeq, TokenSeq, ZigString};

use crate::js::{expr::Seq, values::string::JsString};

pub fn collect_seq<T: Seq>(seq: T) -> Vec<T::Item>
where
    T::Item: Copy,
{
    let slice = unsafe { std::slice::from_raw_parts(seq.data(), seq.len()) };
    slice.to_vec()
}

// pub fn zs_to_str(zs: ZigString) -> &'static str {
//     unsafe {
//         let slice = std::slice::from_raw_parts(zs.data as *const u8, zs.len);
//         let res = std::str::from_utf8_unchecked(slice);
//
//         free_string(zs);
//         res
//     }
// }
//
// pub fn cpseq_to_vec(cpseq: CodePointSeq) -> Vec<CodePoint> {
//     unsafe {
//         let slice = std::slice::from_raw_parts(cpseq.data, cpseq.len);
//         let res = slice.to_vec();
//
//         free_code_point_seq(cpseq);
//         res
//     }
// }

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

    pub fn parse_script(text: ZigString) -> stmt::Script;
}

pub fn utf16_encode_cp_rs(cp: CodePoint) -> String {
    let zs = unsafe { utf16_encode_cp(cp) };
    let s = String::from_utf16(&collect_seq(zs)).unwrap();
    s
}

pub fn cps_to_string_rs(cps: &[CodePoint]) -> String {
    let zs = unsafe { cps_to_string(cps.as_ptr(), cps.len()) };
    let s = String::from_utf16(&collect_seq(zs)).unwrap();
    s
}

pub fn utf16_surrogate_pair_to_cp_rs(high: u16, low: u16) -> CodePoint {
    unsafe { utf16_surrogate_pair_to_cp(high, low) }
}

pub fn code_point_at_rs(s: &str, index: usize) -> CodePointAtResult {
    let u16_vec = s.encode_utf16().collect::<Vec<u16>>();

    let zs = ZigString {
        data: u16_vec.as_ptr(),
        len: u16_vec.len(),
    };
    unsafe { code_point_at(zs, index) }
}

pub fn string_to_cps_rs(text: &str) -> Vec<CodePoint> {
    let u16_vec = text.encode_utf16().collect::<Vec<u16>>();

    let zs = ZigString {
        data: u16_vec.as_ptr(),
        len: u16_vec.len(),
    };
    let cps = unsafe { string_to_cps(zs) };
    let cps_vec = unsafe { std::slice::from_raw_parts(cps.data, cps.len).to_vec() };
    unsafe { free_code_point_seq(cps) };
    cps_vec
}

pub fn parse_text_string_rs(text: &str, goal: u8) -> Vec<expr::Token> {
    let u16_vec = text.encode_utf16().collect::<Vec<u16>>();

    let zs = ZigString {
        data: u16_vec.as_ptr(),
        len: u16_vec.len(),
    };
    let tokens = unsafe { parse_text_string(zs, goal) };
    let tokens_vec = unsafe { std::slice::from_raw_parts(tokens.data, tokens.len).to_vec() };
    unsafe { free_token_seq(tokens) };
    tokens_vec
}

pub fn parse_text_cps_rs(text: &[CodePoint], goal: u8) -> Vec<expr::Token> {
    let tokens = unsafe {
        parse_text_cps(
            CodePointSeq {
                data: text.as_ptr(),
                len: text.len(),
            },
            goal,
        )
    };
    let tokens_vec = unsafe { std::slice::from_raw_parts(tokens.data, tokens.len).to_vec() };
    unsafe { free_token_seq(tokens) };
    tokens_vec
}

pub fn zs_to_js_string(zs: ZigString) -> JsString {
    let s = String::from_utf16(&collect_seq(zs)).unwrap();
    unsafe { free_string(zs) };
    JsString(s.encode_utf16().collect())
}
