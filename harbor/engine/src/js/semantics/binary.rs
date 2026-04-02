use crate::js::{
    expr::{BinaryExpression, BinaryOperator},
    values::ReferenceOrValue,
};

pub fn evaluate(exp: BinaryExpression) -> ReferenceOrValue {
    match exp.operator {
        BinaryOperator::Exponentiation => todo!()
        _ => todo!(),
    }
}
