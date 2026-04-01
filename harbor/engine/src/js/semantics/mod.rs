use crate::js::{
    expr::{
        AssignmentExpression, Expression, IdentifierNameTokenData, LeftHandSideExpression,
        MemberExpression, NewExpression, PrimaryExpression, UnaryExpression,
    },
    values::ReferenceOrValue,
};

pub mod identifier;
pub mod lhs;
pub mod primary;
pub mod unary;

#[derive(Debug, Clone)]
pub enum EvaluateExpressionTag {
    Identifier(IdentifierNameTokenData),
    LeftHandSideExpression(LeftHandSideExpression),
    PrimaryExpression(PrimaryExpression),

    MemberExpression(MemberExpression),
    NewExpression(NewExpression),

    UnaryExpression(UnaryExpression),

    AssignmentExpression(AssignmentExpression),
    Expression(Expression),
}

pub fn general_evaluate(expression: EvaluateExpressionTag) -> ReferenceOrValue {
    match expression {
        EvaluateExpressionTag::Identifier(data) => identifier::evaluate(data),
        EvaluateExpressionTag::LeftHandSideExpression(expr) => lhs::evaluate(expr),
        EvaluateExpressionTag::PrimaryExpression(expr) => primary::evaluate(expr),

        EvaluateExpressionTag::MemberExpression(expr) => lhs::evaluate_member(&expr),
        EvaluateExpressionTag::NewExpression(expr) => lhs::evaluate_new_expr(&expr),

        EvaluateExpressionTag::UnaryExpression(expr) => unary::evaluate(expr),

        _ => todo!("General expression evaluation for tag: {:?}", expression),
    }
}
