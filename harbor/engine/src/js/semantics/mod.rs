use crate::js::{
    expr::{Expression, IdentifierNameTokenData, LeftHandSideExpression, PrimaryExpression},
    values::ReferenceOrValue,
};

pub mod identifier;
pub mod lhs;
pub mod primary;

#[derive(Debug, Clone)]
pub enum EvaluateExpressionTag {
    Identifier(IdentifierNameTokenData),
    LeftHandSideExpression(LeftHandSideExpression),
    PrimaryExpression(PrimaryExpression),

    Expression(Expression),
}

pub fn general_evaluate(expression: EvaluateExpressionTag) -> ReferenceOrValue {
    match expression {
        EvaluateExpressionTag::Identifier(data) => identifier::evaluate(data),
        EvaluateExpressionTag::LeftHandSideExpression(expr) => lhs::evaluate(expr),
        EvaluateExpressionTag::PrimaryExpression(expr) => primary::evaluate(expr),

        _ => todo!("General expression evaluation for tag: {:?}", expression),
    }
}
