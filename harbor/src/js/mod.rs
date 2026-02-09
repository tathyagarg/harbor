use std::fmt::Display;

#[repr(C)]
#[derive(Debug)]
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
}
