use crate::js::{
    expr::{UnaryExpression, UnaryOperator},
    operations::to_number,
    semantics::{EvaluateExpressionTag, general_evaluate},
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
            let mut lhs = general_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.add(&Number(1.0));

            put_value(&mut lhs, Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(old_value));
        }
        UnaryOperator::PostfixDecrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs = general_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.subtract(&Number(1.0));

            put_value(&mut lhs, Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(old_value));
        }
        UnaryOperator::PrefixIncrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs = general_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.add(&Number(1.0));

            put_value(&mut lhs, Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(new_value));
        }
        UnaryOperator::PrefixDecrement => {
            let operand = unsafe { *(*update.operand).data.lhs };
            let mut lhs = general_evaluate(&EvaluateExpressionTag::LeftHandSideExpression(operand));

            let old_value = to_number(get_value(&lhs).unwrap().value).unwrap().value;
            let new_value = old_value.subtract(&Number(1.0));

            put_value(&mut lhs, Value::Number(new_value)).unwrap();

            return ReferenceOrValue::Value(Value::Number(new_value));
        }
        _ => todo!(),
    }
}
