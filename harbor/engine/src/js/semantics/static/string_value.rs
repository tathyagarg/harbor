use crate::js::{expr::IdentifierNameTokenData, values::string::JsString, zs_to_js_string};

/// SS: StringValue
/// https://tc39.es/ecma262/#sec-static-semantics-stringvalue
pub fn string_value(identifier: IdentifierNameTokenData) -> JsString {
    zs_to_js_string(identifier.name)
}
