use crate::js::{
    collect_seq,
    executable::context::resolve_binding,
    expr::{CodePoint, IdentifierNameTokenData},
    string_to_cps,
    values::{ReferenceOrValue, string::JsString},
    zs_to_js_string,
};

/// SS: IdentifierCodePoints
/// https://tc39.es/ecma262/#sec-identifiercodepoints
pub fn identifier_code_points(identifier: IdentifierNameTokenData) -> Vec<CodePoint> {
    collect_seq(unsafe { string_to_cps(identifier.name) })
}

/// SS: StringValue
/// https://tc39.es/ecma262/#sec-static-semantics-stringvalue
pub fn string_value(identifier: IdentifierNameTokenData) -> JsString {
    zs_to_js_string(identifier.name)
}

/// RS: Evaluation
/// https://tc39.es/ecma262/#sec-identifiers-runtime-semantics-evaluation
pub fn evaluate(identifier: &IdentifierNameTokenData) -> ReferenceOrValue {
    ReferenceOrValue::Reference(
        resolve_binding(JsString(collect_seq(identifier.name)), None)
            .unwrap()
            .value,
    )
}
