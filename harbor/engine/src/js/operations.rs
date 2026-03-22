use std::str::FromStr;

use crate::js::{
    types::completion_record::{
        CompletionRecord, CompletionRecordError, CompletionRecordNormal, CompletionRecordThrow,
    },
    values::{
        Value,
        number::Number,
        string::{_equals_raw, JsString},
    },
};

pub fn to_number(
    argument: Value,
) -> Result<CompletionRecord<Number>, CompletionRecord<CompletionRecordError>> {
    match argument {
        Value::Number(n) => return Ok(CompletionRecordNormal(n)),
        Value::Symbol(_) | Value::BigInt(_) => {
            return Err(CompletionRecordThrow(CompletionRecordError::TypeError));
        }
        Value::Undefined => return Ok(CompletionRecordNormal(Number(f64::NAN))),
        Value::Null | Value::Boolean(false) => {
            return Ok(CompletionRecordNormal(Number(0.0)));
        }
        Value::Boolean(true) => return Ok(CompletionRecordNormal(Number(1.0))),
        Value::String(s) => return Ok(CompletionRecordNormal(string_to_number(s))),
        _ => todo!("to_number for object"),
    }
}

pub fn string_to_number(argument: JsString) -> Number {
    if _equals_raw(&argument, "Infinity") || _equals_raw(&argument, "+Infinity") {
        return Number(f64::INFINITY);
    } else if _equals_raw(&argument, "-Infinity") {
        return Number(f64::NEG_INFINITY);
    } else if _equals_raw(&argument, "NaN") {
        return Number(f64::NAN);
    } else {
        let str = String::from_utf16(&argument.0).unwrap();
        return Number(str::parse::<f64>(&str).unwrap_or(f64::NAN));
    }
}

pub fn to_int32(
    argument: Value,
) -> Result<CompletionRecord<Number>, CompletionRecord<CompletionRecordError>> {
    let number = to_number(argument)?.value;

    if number.0.is_infinite() || number.0 == 0.0 {
        return Ok(CompletionRecordNormal(Number(0.0)));
    }

    let int = number.0.trunc() % f64::powi(2.0, 32);
    if int >= f64::powi(2.0, 31) {
        return Ok(CompletionRecordNormal(Number(int - f64::powi(2.0, 32))));
    }

    return Ok(CompletionRecordNormal(Number(int)));
}

pub fn to_uint32(
    argument: Value,
) -> Result<CompletionRecord<Number>, CompletionRecord<CompletionRecordError>> {
    let number = to_number(argument)?.value;

    if number.0.is_infinite() || number.0 == 0.0 {
        return Ok(CompletionRecordNormal(Number(0.0)));
    }

    let int = number.0.trunc() % f64::powi(2.0, 32);

    return Ok(CompletionRecordNormal(Number(int)));
}

pub fn to_string(
    argument: Value,
) -> Result<CompletionRecord<JsString>, CompletionRecord<CompletionRecordError>> {
    match argument {
        Value::String(s) => return Ok(CompletionRecordNormal(s)),
        Value::Symbol(_) => {
            return Err(CompletionRecordThrow(CompletionRecordError::TypeError));
        }
        Value::Undefined => {
            return Ok(CompletionRecordNormal(
                JsString::from_str("undefined").unwrap(),
            ));
        }
        Value::Null => return Ok(CompletionRecordNormal(JsString::from_str("null").unwrap())),
        Value::Boolean(b) => {
            if b {
                return Ok(CompletionRecordNormal(JsString::from_str("true").unwrap()));
            } else {
                return Ok(CompletionRecordNormal(JsString::from_str("false").unwrap()));
            }
        }
        Value::Number(n) => return Ok(CompletionRecordNormal(n.to_string(10))),
        _ => todo!("to_string for object"),
    }
}

pub fn canonical_numeric_index_string(argument: &JsString) -> Option<Number> {
    if *argument == "-0" {
        return Some(Number(-0.0));
    }

    let n = to_number(Value::String(argument.clone()));
    match n {
        Err(_) => None,
        Ok(CompletionRecord { value, .. }) => {
            if to_string(Value::Number(value)).unwrap().value == *argument {
                Some(value)
            } else {
                None
            }
        }
    }
}
