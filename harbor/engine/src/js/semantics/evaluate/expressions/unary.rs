use crate::js::{
    expr::{UNARY_EXPR_OR_LHS_LHS, UNARY_EXPR_OR_LHS_UNARY, UnaryExpression, UnaryOperator},
    operations::to_number,
    semantics::evaluate::expressions::{EvaluateExpressionTag, expression_evaluate},
    values::{
        ReferenceOrValue, Value,
        number::Number,
        reference::{get_value, put_value},
    },
};

pub fn evaluate(update: &UnaryExpression) -> ReferenceOrValue {
    match update.operator {
        UnaryOperator::PostfixIncrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs =
                expression_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.add(&Number(1.0));

            put_value(&mut lhs, &Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(old_value));
        }
        UnaryOperator::PostfixDecrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs =
                expression_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.subtract(&Number(1.0));

            put_value(&mut lhs, &Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(old_value));
        }
        UnaryOperator::PrefixIncrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs =
                expression_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.add(&Number(1.0));

            put_value(&mut lhs, &Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(new_value));
        }
        UnaryOperator::PrefixDecrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs =
                expression_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.subtract(&Number(1.0));

            put_value(&mut lhs, &Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(new_value));
        }
        UnaryOperator::None => {
            let operand = unsafe { *update.operand };
            match operand.tag {
                UNARY_EXPR_OR_LHS_LHS => {
                    let lhs = unsafe { *operand.data.lhs };
                    return super::lhs::evaluate(&lhs);
                }
                UNARY_EXPR_OR_LHS_UNARY => {
                    let unary = unsafe { *operand.data.unary };
                    return evaluate(&unary);
                }
                _ => unreachable!("Unknown operand tag for unary expression: {}", operand.tag),
            }
        }
        _ => unreachable!("Unknown unary operator: {:?}", update.operator),
    }
}
