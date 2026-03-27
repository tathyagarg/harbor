use crate::js::{
    executable::environment::EnvironmentRecord,
    values::{number::Number, object::Object, string::JsString, symbol::Symbol},
};

pub mod object;

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

pub mod number {
    use std::{ops::Add, str::FromStr};

    use crate::js::{
        operations::{to_int32, to_uint32},
        values::{Value, string::JsString},
    };

    pub enum BitwiseOp {
        And,
        Xor,
        Or,
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Number(pub f64);

    impl Add<f64> for Number {
        type Output = Self;

        fn add(self, rhs: f64) -> Self::Output {
            Number(self.0 + rhs)
        }
    }

    impl Number {
        pub fn unary_minus(&self) -> Self {
            Number(if self.0.is_nan() { f64::NAN } else { -self.0 })
        }

        pub fn bitwise_not(&self) -> Self {
            let old_value = to_int32(Value::Number(*self)).unwrap().value.0 as i32;
            return Number((!old_value) as f64);
        }

        /// NOTE: This may not be spec-compliant
        pub fn exponentiate(&self, other: &Number) -> Self {
            Number(self.0.powf(other.0))
        }

        pub fn multiply(&self, other: &Number) -> Self {
            Number(self.0 * other.0)
        }

        pub fn divide(&self, other: &Number) -> Self {
            Number(self.0 / other.0)
        }

        pub fn remainder(&self, other: &Number) -> Self {
            Number(self.0 % other.0)
        }

        pub fn add(&self, other: &Number) -> Self {
            Number(self.0 + other.0)
        }

        pub fn subtract(&self, other: &Number) -> Self {
            Number(self.0 - other.0)
        }

        pub fn left_shift(&self, other: &Number) -> Self {
            let left = to_int32(Value::Number(*self)).unwrap().value.0 as i32;
            let right = to_uint32(Value::Number(*other)).unwrap().value.0 as u32;
            let shift_count = right % 32;

            Number(((left << shift_count) as i32) as f64)
        }

        pub fn signed_right_shift(&self, other: &Number) -> Self {
            let left = to_int32(Value::Number(*self)).unwrap().value.0 as i32;
            let right = to_uint32(Value::Number(*other)).unwrap().value.0 as u32;
            let shift_count = right % 32;

            Number((left >> shift_count) as f64)
        }

        pub fn unsigned_right_shift(&self, other: &Number) -> Self {
            let left = to_uint32(Value::Number(*self)).unwrap().value.0 as u32;
            let right = to_uint32(Value::Number(*other)).unwrap().value.0 as u32;
            let shift_count = right % 32;

            Number((left >> shift_count) as f64)
        }

        pub fn less_than(&self, other: &Number) -> bool {
            self.0 < other.0
        }

        pub fn equal(&self, other: &Number) -> bool {
            self.0 == other.0
        }

        pub fn same_value(&self, other: &Number) -> bool {
            if self.0.is_nan() && other.0.is_nan() {
                return true;
            }

            if self.0 == 0.0 && other.0 == 0.0 {
                return self.0.is_sign_negative() == other.0.is_sign_negative();
            }

            self.0 == other.0
        }

        pub fn same_value_zero(&self, other: &Number) -> bool {
            if self.0.is_nan() && other.0.is_nan() {
                return true;
            }

            self.0 == other.0
        }

        pub fn bitwise_and(&self, other: &Number) -> Self {
            number_bitwise_op(BitwiseOp::And, self, other)
        }

        pub fn bitwise_xor(&self, other: &Number) -> Self {
            number_bitwise_op(BitwiseOp::Xor, self, other)
        }

        pub fn bitwise_or(&self, other: &Number) -> Self {
            number_bitwise_op(BitwiseOp::Or, self, other)
        }

        /// NOTE: This may not be spec-compliant
        pub fn to_string(&self, radix: u8) -> JsString {
            if self.0.is_nan() {
                return JsString::from_str("NaN").unwrap();
            }

            if self.0 == 0.0 {
                return JsString::from_str("0").unwrap();
            }

            if self.0 < 0.0 {
                return JsString::from_str("-").unwrap() + Number(-self.0).to_string(radix);
            }

            if self.0.is_infinite() {
                return JsString::from_str("Infinity").unwrap();
            }

            return JsString::from_str(&self.0.to_string().to_uppercase()).unwrap();
        }
    }

    pub fn number_bitwise_op(op: BitwiseOp, left: &Number, right: &Number) -> Number {
        let left_int = to_int32(Value::Number(*left)).unwrap().value.0 as i32;
        let right_int = to_int32(Value::Number(*right)).unwrap().value.0 as i32;

        let result = match op {
            BitwiseOp::And => left_int & right_int,
            BitwiseOp::Xor => left_int ^ right_int,
            BitwiseOp::Or => left_int | right_int,
        };

        Number(result as f64)
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
}

#[derive(Clone, Debug)]
pub enum ReferenceBase {
    Value(Value),
    EnvironmentRecord(EnvironmentRecord),
    Unresolvable,
}

#[derive(Clone, Debug)]
pub struct Reference {
    pub base: ReferenceBase,
    pub referenced_name: Value,
    pub strict: bool,
    pub this_value: Option<Value>,
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
