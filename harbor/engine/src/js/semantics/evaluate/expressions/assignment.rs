use crate::js::{
    expr::{
        ASSIGNMENT_EXPR_BINARY, ASSIGNMENT_EXPR_LHS, ASSIGNMENT_EXPR_OPERATOR,
        ASSIGNMENT_EXPR_PRIMARY, ASSIGNMENT_EXPR_RAW, ASSIGNMENT_EXPR_UNARY, AssignmentExpression,
        AssignmentOperator, BinaryOperator,
    },
    operations::to_boolean,
    semantics::expressions::apply_string_or_numeric_binary_operator,
    values::{ReferenceOrValue, reference::put_value},
};

pub fn evaluate(exp: &AssignmentExpression) -> ReferenceOrValue {
    match exp.tag {
        ASSIGNMENT_EXPR_RAW => {
            let left_raw = unsafe { *exp.data.raw_assignment.left };
            let mut left_ref = super::lhs::evaluate(&left_raw);

            let right_raw = unsafe { *exp.data.raw_assignment.right };
            let right_ref = evaluate(&right_raw);
            let right_val = right_ref.get_value().unwrap().value;

            put_value(&mut left_ref, &right_val).unwrap();
            return ReferenceOrValue::Value(right_val);
        }
        ASSIGNMENT_EXPR_OPERATOR => {
            let op_assignment = unsafe { exp.data.operator_assignment };

            let left_raw = unsafe { *op_assignment.left };
            let mut left_ref = super::lhs::evaluate(&left_raw);
            let left_val = left_ref.get_value().unwrap().value;

            let operator = match op_assignment.operator {
                AssignmentOperator::Plus => BinaryOperator::Plus,
                AssignmentOperator::Minus => BinaryOperator::Minus,
                AssignmentOperator::Star => BinaryOperator::Star,
                AssignmentOperator::Slash => BinaryOperator::Slash,
                AssignmentOperator::Percent => BinaryOperator::Percent,
                AssignmentOperator::Exponentiation => BinaryOperator::Exponentiation,
                AssignmentOperator::LeftShift => BinaryOperator::LeftShift,
                AssignmentOperator::RightShift => BinaryOperator::RightShift,
                AssignmentOperator::UnsignedRightShift => BinaryOperator::UnsignedRightShift,
                AssignmentOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
                AssignmentOperator::BitwiseXor => BinaryOperator::BitwiseXor,
                AssignmentOperator::BitwiseOr => BinaryOperator::BitwiseOr,
                AssignmentOperator::ShortCircuitLogicalAnd => {
                    if !to_boolean(&left_val) {
                        return ReferenceOrValue::Value(left_val);
                    }

                    let right_raw = unsafe { *op_assignment.right };
                    let right_ref = evaluate(&right_raw);
                    let right_val = right_ref.get_value().unwrap().value;

                    put_value(&mut left_ref, &right_val).unwrap();
                    return ReferenceOrValue::Value(right_val);
                }
                AssignmentOperator::ShortCircuitLogicalOr => BinaryOperator::LogicalOr,
                AssignmentOperator::NullishCoalescing => BinaryOperator::NullishCoalescing,
                AssignmentOperator::Raw => panic!("This is reachable?"),
            };

            let right_raw = unsafe { *op_assignment.right };
            let right_ref = evaluate(&right_raw);
            let right_val = right_ref.get_value().unwrap().value;

            let r = apply_string_or_numeric_binary_operator(&left_val, &right_val, operator);
            put_value(&mut left_ref, &r.get_value().unwrap().value).unwrap();

            return r;
        }
        ASSIGNMENT_EXPR_BINARY => {
            let binary_data = unsafe { *exp.data.binary };
            return super::binary::evaluate(&binary_data);
        }
        ASSIGNMENT_EXPR_LHS => {
            let lhs_data = unsafe { *exp.data.lhs };
            return super::lhs::evaluate(&lhs_data);
        }
        ASSIGNMENT_EXPR_PRIMARY => {
            let primary_data = unsafe { *exp.data.primary };
            return super::primary::evaluate(&primary_data);
        }
        ASSIGNMENT_EXPR_UNARY => {
            let unary_data = unsafe { *exp.data.unary };
            return super::unary::evaluate(&unary_data);
        }
        _ => unreachable!("Unknown assignment expression tag: {}", exp.tag),
    }
}
