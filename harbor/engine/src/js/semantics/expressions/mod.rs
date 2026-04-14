use crate::js::{
    collect_seq,
    expr::{
        AssignmentExpression, BinaryExpression, BinaryOperator, Expression,
        IdentifierNameTokenData, LeftHandSideExpression, MemberExpression, NewExpression,
        PrimaryExpression, UnaryExpression,
    },
    operations::{to_number, to_primitive, to_string},
    values::{ReferenceOrValue, Value, number::Number},
};

pub mod assignment;
pub mod binary;
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
    BinaryExpression(BinaryExpression),

    AssignmentExpression(AssignmentExpression),
    Expression(Expression),
}

pub fn apply_string_or_numeric_binary_operator(
    left: &Value,
    right: &Value,
    operator: BinaryOperator,
) -> ReferenceOrValue {
    let (l_val, r_val) = if operator == BinaryOperator::Plus {
        let wrapped_l_prim = to_primitive(&left).unwrap();
        let wrapped_r_prim = to_primitive(&right).unwrap();

        let l_prim = wrapped_l_prim.value;
        let r_prim = wrapped_r_prim.value;

        if l_prim.is_string() || r_prim.is_string() {
            let wrapped_l_str = to_string(&l_prim).unwrap();
            let wrapped_r_str = to_string(&r_prim).unwrap();

            let l_str = wrapped_l_str.unwrapped();
            let r_str = wrapped_r_str.unwrapped();

            return ReferenceOrValue::Value(Value::String(l_str.concat(r_str)));
        }

        (l_prim, r_prim)
    } else {
        (left.clone(), right.clone())
    };

    // NOTE: This should be to_numeric, not to_number
    // But since we don't have BigInt yet, to_numeric is the same as to_number for now

    let l_num = to_number(l_val).unwrap().value;
    let r_num = to_number(r_val).unwrap().value;

    let operation = match operator {
        BinaryOperator::Plus => Number::add,
        BinaryOperator::Minus => Number::subtract,
        BinaryOperator::Star => Number::multiply,
        BinaryOperator::Slash => Number::divide,
        BinaryOperator::Percent => Number::remainder,
        BinaryOperator::Exponentiation => Number::exponentiate,
        BinaryOperator::LeftShift => Number::left_shift,
        BinaryOperator::RightShift => Number::signed_right_shift,
        BinaryOperator::UnsignedRightShift => Number::unsigned_right_shift,
        BinaryOperator::BitwiseAnd => Number::bitwise_and,
        BinaryOperator::BitwiseXor => Number::bitwise_xor,
        BinaryOperator::BitwiseOr => Number::bitwise_or,
        _ => unreachable!(),
    };

    return ReferenceOrValue::Value(Value::Number(operation(&l_num, &r_num)));
}

pub fn eval_string_or_numeric_bin_expr(
    left: &EvaluateExpressionTag,
    right: &EvaluateExpressionTag,
    operator: BinaryOperator,
) -> ReferenceOrValue {
    let left_ref = general_evaluate(left);
    let right_ref = general_evaluate(right);

    let left_val = left_ref.get_value().unwrap().value;
    let right_val = right_ref.get_value().unwrap().value;

    apply_string_or_numeric_binary_operator(&left_val, &right_val, operator)
}

pub fn general_evaluate(expression: &EvaluateExpressionTag) -> ReferenceOrValue {
    match expression {
        EvaluateExpressionTag::Identifier(data) => identifier::evaluate(data),
        EvaluateExpressionTag::LeftHandSideExpression(expr) => lhs::evaluate(expr),
        EvaluateExpressionTag::PrimaryExpression(expr) => primary::evaluate(expr),

        EvaluateExpressionTag::MemberExpression(expr) => lhs::evaluate_member(&expr),
        EvaluateExpressionTag::NewExpression(expr) => lhs::evaluate_new_expr(&expr),

        EvaluateExpressionTag::UnaryExpression(expr) => unary::evaluate(expr),
        EvaluateExpressionTag::BinaryExpression(expr) => binary::evaluate(expr),

        EvaluateExpressionTag::AssignmentExpression(expr) => assignment::evaluate(expr),

        EvaluateExpressionTag::Expression(expr) => {
            let exprs = collect_seq(expr);
            for e in exprs[..exprs.len() - 1].iter() {
                general_evaluate(&EvaluateExpressionTag::AssignmentExpression(e.clone()));
            }

            let right_expr = exprs.last().unwrap();

            general_evaluate(&EvaluateExpressionTag::AssignmentExpression(
                right_expr.clone(),
            ))
        }
    }
}
