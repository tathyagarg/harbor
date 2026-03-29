use crate::js::values::{
    number::Number, object::Object, reference::Reference, string::JsString, symbol::Symbol,
};

pub mod number;
pub mod object;

pub mod reference;

pub enum ReferenceOrValue {
    Reference(Reference),
    Value(Value),
}

pub mod string {
    use std::{ops::Add, str::FromStr};

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct JsString(pub Vec<u16>);

    impl FromStr for JsString {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(JsString(s.encode_utf16().collect()))
        }
    }

    impl Add for JsString {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            let mut combined = self.0;
            combined.extend(rhs.0);
            JsString(combined)
        }
    }

    impl PartialEq<&str> for JsString {
        fn eq(&self, other: &&str) -> bool {
            let other_utf16: Vec<u16> = other.encode_utf16().collect();
            self.0 == other_utf16
        }

        fn ne(&self, other: &&str) -> bool {
            !self.eq(other)
        }
    }

    pub fn string_index_of(
        string: &JsString,
        search_value: &JsString,
        from_index: usize,
    ) -> Option<usize> {
        let len = string.0.len();

        if search_value.0.is_empty() && from_index <= len {
            return Some(from_index);
        }

        let search_len = search_value.0.len();

        if search_len + from_index > len {
            return None;
        }

        for i in from_index..=len - search_len {
            let candidate = &string.0[i..i + search_len];
            if candidate == search_value.0.as_slice() {
                return Some(i);
            }
        }

        None
    }

    pub fn string_last_index_of(
        string: &JsString,
        search_value: &JsString,
        from_index: usize,
    ) -> Option<usize> {
        let search_len = search_value.0.len();

        for i in (0..=from_index).rev() {
            let candidate = &string.0[i..i + search_len];
            if candidate == search_value.0.as_slice() {
                return Some(i);
            }
        }

        None
    }

    pub fn _equals_raw(string1: &JsString, string2: &str) -> bool {
        let string2_utf16: Vec<u16> = string2.encode_utf16().collect();
        string1.0 == string2_utf16
    }
}

pub mod symbol {
    use crate::js::values::string::JsString;

    pub type SymbolId = u64;

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct Symbol {
        pub id: SymbolId,
        pub description: Option<JsString>,
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    String(JsString),
    Symbol(Symbol),
    Number(Number),
    BigInt(()),
    Object(Object),
}

impl Value {
    pub fn empty() -> Self {
        Value::Undefined
    }

    pub fn unwrap_bool(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn unwrap_number(&self) -> Option<Number> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn unwrap_object(&self) -> Option<Object> {
        if let Value::Object(o) = self {
            Some(o.clone())
        } else {
            None
        }
    }

    pub fn unwrap_string(&self) -> Option<JsString> {
        if let Value::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn unwrap_symbol(&self) -> Option<Symbol> {
        if let Value::Symbol(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn is_property_key(&self) -> bool {
        matches!(self, Value::String(_) | Value::Symbol(_))
    }
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
