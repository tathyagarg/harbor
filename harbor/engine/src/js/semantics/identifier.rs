use crate::js::{
    collect_seq,
    expr::{CodePoint, IdentifierNameTokenData},
    string_to_cps,
};

/// SS: IdentifierCodePoints
/// https://tc39.es/ecma262/#sec-identifiercodepoints
pub fn identifier_code_points(identifier: IdentifierNameTokenData) -> Vec<CodePoint> {
    collect_seq(unsafe { string_to_cps(identifier.name) })
}

/// SS: StringValue
/// https://tc39.es/ecma262/#sec-static-semantics-stringvalue
pub fn string_value(identifier: IdentifierNameTokenData) -> String {
    let cps = identifier_code_points(identifier);
    String::from_utf16(&cps.iter().map(|cp| *cp as u16).collect::<Vec<u16>>()).unwrap()
}
