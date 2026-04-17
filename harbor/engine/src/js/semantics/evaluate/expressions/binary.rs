use crate::js::{
    expr::{
        BINARY_OR_UNARY_EXPR_BINARY, BINARY_OR_UNARY_EXPR_UNARY, BinaryExpression, BinaryOperator,
        UNARY_EXPR_OR_NULL_UNARY,
    },
    operations::{is_less_than, is_loosely_equal, is_strictly_equal, to_boolean},
    semantics::expressions::{
        EvaluateExpressionTag, eval_string_or_numeric_bin_expr, expression_evaluate,
    },
    values::{ReferenceOrValue, Value},
};

pub fn evaluate(exp: &BinaryExpression) -> ReferenceOrValue {
    let left_raw = unsafe { *exp.left };
    let right_raw = unsafe { *exp.right };

    let left = if left_raw.tag == BINARY_OR_UNARY_EXPR_BINARY {
        EvaluateExpressionTag::BinaryExpression(unsafe { *left_raw.data.binary })
    } else if left_raw.tag == BINARY_OR_UNARY_EXPR_UNARY {
        EvaluateExpressionTag::UnaryExpression(unsafe { *left_raw.data.unary })
    } else {
        panic!(
            "Invalid left operand for exponentiation operator: {:?}",
            left_raw
        );
    };

    let right = if right_raw.tag == UNARY_EXPR_OR_NULL_UNARY {
        EvaluateExpressionTag::UnaryExpression(unsafe { *right_raw.data.unary })
    } else {
        panic!(
            "Invalid right operand for exponentiation operator: {:?}",
            right_raw
        );
    };

    match exp.operator {
        BinaryOperator::Exponentiation
        | BinaryOperator::Star
        | BinaryOperator::Slash
        | BinaryOperator::Percent
        | BinaryOperator::Plus
        | BinaryOperator::Minus
        | BinaryOperator::LeftShift
        | BinaryOperator::RightShift
        | BinaryOperator::UnsignedRightShift
        | BinaryOperator::BitwiseAnd
        | BinaryOperator::BitwiseXor
        | BinaryOperator::BitwiseOr => eval_string_or_numeric_bin_expr(&left, &right, exp.operator),
        BinaryOperator::LogicalAnd => {
            let left_ref = expression_evaluate(&left);
            let left_val = left_ref.get_value().unwrap().value;

            if !to_boolean(&left_val) {
                return ReferenceOrValue::Value(left_val);
            }

            let right_ref = expression_evaluate(&right);
            let right_val = right_ref.get_value().unwrap().value;

            ReferenceOrValue::Value(right_val)
        }
        BinaryOperator::LogicalOr => {
            let left_ref = expression_evaluate(&left);
            let left_val = left_ref.get_value().unwrap().value;

            if to_boolean(&left_val) {
                return ReferenceOrValue::Value(left_val);
            }

            let right_ref = expression_evaluate(&right);
            let right_val = right_ref.get_value().unwrap().value;

            ReferenceOrValue::Value(right_val)
        }
        BinaryOperator::NullishCoalescing => {
            let left_ref = expression_evaluate(&left);
            let left_val = left_ref.get_value().unwrap().value;

            if !left_val.is_null() && !left_val.is_undefined() {
                return ReferenceOrValue::Value(left_val);
            }

            let right_ref = expression_evaluate(&right);
            let right_val = right_ref.get_value().unwrap().value;

            ReferenceOrValue::Value(right_val)
        }
        _ => {
            let left_ref = expression_evaluate(&left);
            let right_ref = expression_evaluate(&right);

            let left_val = left_ref.get_value().unwrap().value;
            let right_val = right_ref.get_value().unwrap().value;

            match exp.operator {
                BinaryOperator::LessThan => {
                    let r = is_less_than(&left_val, &right_val).value;
                    if let Some(r_val) = r {
                        ReferenceOrValue::Value(Value::Boolean(r_val))
                    } else {
                        ReferenceOrValue::Value(Value::Boolean(false))
                    }
                }
                BinaryOperator::GreaterThan => {
                    let r = is_less_than(&right_val, &left_val).value;
                    if let Some(r_val) = r {
                        ReferenceOrValue::Value(Value::Boolean(r_val))
                    } else {
                        ReferenceOrValue::Value(Value::Boolean(false))
                    }
                }
                BinaryOperator::LessThanOrEqual => {
                    let r = is_less_than(&right_val, &left_val).value;
                    if let Some(r_val) = r {
                        ReferenceOrValue::Value(Value::Boolean(!r_val))
                    } else {
                        ReferenceOrValue::Value(Value::Boolean(false))
                    }
                }
                BinaryOperator::GreaterThanOrEqual => {
                    let r = is_less_than(&left_val, &right_val).value;
                    if let Some(r_val) = r {
                        ReferenceOrValue::Value(Value::Boolean(!r_val))
                    } else {
                        ReferenceOrValue::Value(Value::Boolean(false))
                    }
                }
                BinaryOperator::Equal => {
                    let r = is_loosely_equal(&right_val, &left_val).unwrap().value;
                    ReferenceOrValue::Value(Value::Boolean(r))
                }
                BinaryOperator::NotEqual => {
                    let r = is_loosely_equal(&right_val, &left_val).unwrap().value;
                    ReferenceOrValue::Value(Value::Boolean(!r))
                }
                BinaryOperator::StrictEqual => {
                    let r = is_strictly_equal(&right_val, &left_val);
                    ReferenceOrValue::Value(Value::Boolean(r))
                }
                BinaryOperator::StrictNotEqual => {
                    let r = is_strictly_equal(&right_val, &left_val);
                    ReferenceOrValue::Value(Value::Boolean(!r))
                }
                _ => todo!(),
            }
        }
    }
}
