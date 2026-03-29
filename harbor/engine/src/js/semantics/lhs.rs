use crate::js::{
    expr::{
        LEFT_HAND_SIDE_EXPR_CALL, LEFT_HAND_SIDE_EXPR_NEW, LeftHandSideExpression,
        MEMBER_EXPR_MEMBER, MEMBER_EXPR_PRIMARY, MemberExpression, NEW_EXPR_MEMBER,
    },
    semantics::primary,
    values::{ReferenceOrValue, Value, reference::get_value},
};

pub fn evaluate(lhs: LeftHandSideExpression) -> Value {
    match lhs.tag {
        LEFT_HAND_SIDE_EXPR_NEW => {
            let new_data = unsafe { *lhs.data.new };
            match new_data.tag {
                NEW_EXPR_MEMBER => {
                    let member_data = unsafe { *new_data.data.member };
                    return _evaluate_member(&member_data);
                }
                _ => unreachable!("Unknown new expression tag: {}", new_data.tag),
            }
        }
        LEFT_HAND_SIDE_EXPR_CALL => {
            let call_data = unsafe { *lhs.data.call };
            todo!(
                "Implement left-hand side call expression evaluation: {:?}",
                call_data
            );
        }
        _ => unreachable!("Unknown left-hand side expression tag: {}", lhs.tag),
    }
}

fn _evaluate_member(member: &MemberExpression) -> Value {
    match member.tag {
        MEMBER_EXPR_PRIMARY => {
            let primary_data = unsafe { *member.data.primary };
            return primary::evaluate(primary_data);
        }
        MEMBER_EXPR_MEMBER => {
            let object_data = unsafe { *member.data.member.object };
            let expression_data = unsafe { *member.data.member.expr };

            let base_reference = _evaluate_member(&object_data);
            let base_value = get_value(ReferenceOrValue::Value(base_reference));

            todo!()
        }
        _ => unreachable!("Unknown member expression tag: {}", member.tag),
    }
}
