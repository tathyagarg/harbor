use crate::js::values::Value;

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
