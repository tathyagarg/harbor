use crate::js::{
    expr::{
        BINARY_OR_UNARY_EXPR_BINARY, BINARY_OR_UNARY_EXPR_UNARY, BinaryExpression, BinaryOperator,
        UNARY_EXPR_OR_NULL_UNARY,
    },
    operations::{is_less_than, is_loosely_equal},
    semantics::{EvaluateExpressionTag, eval_string_or_numeric_bin_expr, general_evaluate},
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

    let left_ref = general_evaluate(&left);
    let right_ref = general_evaluate(&right);

    let left_val = left_ref.get_value().unwrap().value;
    let right_val = right_ref.get_value().unwrap().value;

    match exp.operator {
        BinaryOperator::Exponentiation
        | BinaryOperator::Star
        | BinaryOperator::Slash
        | BinaryOperator::Percent
        | BinaryOperator::Plus
        | BinaryOperator::Minus
        | BinaryOperator::LeftShift
        | BinaryOperator::RightShift
        | BinaryOperator::UnsignedRightShift => {
            eval_string_or_numeric_bin_expr(&left, &right, exp.operator)
        }
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
        _ => todo!(),
    }
}
