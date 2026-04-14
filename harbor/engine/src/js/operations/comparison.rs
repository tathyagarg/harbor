use crate::js::{
    operations::{to_number, to_primitive},
    types::completion_record::{CRKThrow, CompletionRecord, CompletionRecordNormal},
    values::{Value, number::Number},
};

pub fn is_callable(arg: &Value) -> bool {
    if let Value::Object(obj) = arg {
        todo!("Check if object is callable");
    }

    return false;
}

pub fn is_constructor(arg: &Value) -> bool {
    if let Value::Object(obj) = arg {
        todo!("Check if object is a constructor");
    }

    return false;
}

pub fn same_type(x: &Value, y: &Value) -> bool {
    match (x, y) {
        (Value::Undefined, Value::Undefined)
        | (Value::Null, Value::Null)
        | (Value::Boolean(_), Value::Boolean(_))
        | (Value::String(_), Value::String(_))
        | (Value::Symbol(_), Value::Symbol(_))
        | (Value::Number(_), Value::Number(_))
        | (Value::BigInt(_), Value::BigInt(_))
        | (Value::Object(_), Value::Object(_)) => true,
        _ => false,
    }
}

pub fn same_value(x: &Value, y: &Value) -> bool {
    if !same_type(x, y) {
        return false;
    }

    if let Value::Number(x_num) = x
        && let Value::Number(y_num) = y
    {
        return x_num.same_value(y_num);
    }

    return same_value_non_number(x, y);
}

pub fn same_value_zero(x: &Value, y: &Value) -> bool {
    if !same_type(x, y) {
        return false;
    }

    if let Value::Number(x_num) = x
        && let Value::Number(y_num) = y
    {
        return x_num.same_value_zero(y_num);
    }

    return same_value_non_number(x, y);
}

pub fn same_value_non_number(x: &Value, y: &Value) -> bool {
    if let Value::Undefined = x {
        return true;
    }
    if let Value::Null = x {
        return true;
    }

    if let Value::String(x_str) = x
        && let Value::String(y_str) = y
    {
        return x_str == y_str;
    }

    if let Value::Boolean(x_bool) = x
        && let Value::Boolean(y_bool) = y
    {
        return x_bool == y_bool;
    }

    return false;
}

pub fn is_less_than(x: &Value, y: &Value) -> CompletionRecord<Option<bool>> {
    let (wrapped_px, wrapped_py) = (to_primitive(x).unwrap(), to_primitive(y).unwrap());

    let (px, py) = (wrapped_px.value, wrapped_py.value);

    if px.is_string() && py.is_string() {
        let px_string = px.unwrap_string().unwrap();
        let py_string = py.unwrap_string().unwrap();

        let lx = px_string.len();
        let ly = py_string.len();

        for i in 0..lx.min(ly) {
            let cx = px_string.code_point_at(i).unwrap();
            let cy = py_string.code_point_at(i).unwrap();

            if cx < cy {
                return CompletionRecordNormal(Some(true));
            }
            if cx > cy {
                return CompletionRecordNormal(Some(false));
            }
        }

        if lx < ly {
            return CompletionRecordNormal(Some(true));
        }

        return CompletionRecordNormal(Some(false));
    }

    let nx = to_number(px).unwrap().value;
    let ny = to_number(py).unwrap().value;

    let res = Number::less_than(&nx, &ny);
    CompletionRecordNormal(Some(res))
}

pub fn is_loosely_equal(
    x: &Value,
    y: &Value,
) -> Result<CompletionRecord<bool>, CompletionRecord<(), CRKThrow>> {
    if same_type(x, y) {
        let res = is_strictly_equal(x, y);
        return Ok(CompletionRecordNormal(res));
    }

    if (x.is_null() && y.is_undefined()) || (x.is_undefined() && y.is_null()) {
        return Ok(CompletionRecordNormal(true));
    }

    if x.is_number() && y.is_string() {
        let y_num = to_number(y.clone()).unwrap().value;
        return is_loosely_equal(x, &Value::Number(y_num));
    }

    if x.is_string() && y.is_number() {
        let x_num = to_number(x.clone()).unwrap().value;
        return is_loosely_equal(&Value::Number(x_num), y);
    }

    if x.is_boolean() {
        let x_num = to_number(x.clone()).unwrap().value;
        return is_loosely_equal(&Value::Number(x_num), y);
    }

    if y.is_boolean() {
        let y_num = to_number(y.clone()).unwrap().value;
        return is_loosely_equal(x, &Value::Number(y_num));
    }

    if (x.is_string() || x.is_number() || x.is_symbol()) && y.is_object() {
        let y_prim = to_primitive(y).unwrap().value;
        return is_loosely_equal(x, &y_prim);
    }

    if x.is_object() && (y.is_string() || y.is_number() || y.is_symbol()) {
        let x_prim = to_primitive(x).unwrap().value;
        return is_loosely_equal(&x_prim, y);
    }

    Ok(CompletionRecordNormal(false))
}

pub fn is_strictly_equal(x: &Value, y: &Value) -> bool {
    if !same_type(x, y) {
        return false;
    }

    if x.is_number() {
        let x_num = x.unwrap_number().unwrap();
        let y_num = y.unwrap_number().unwrap();
        return Number::same_value(&x_num, &y_num);
    }

    same_value_non_number(x, y)
}
