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
