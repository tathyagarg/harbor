use crate::js::{
    collect_seq,
    expr::{
        Arguments, CALL_EXPR_MEMBER, CALL_EXPR_PRIVATE_PROPERTY, CALL_EXPR_PROPERTY,
        CallExpression, IdentifierNameTokenData, LEFT_HAND_SIDE_EXPR_CALL, LEFT_HAND_SIDE_EXPR_NEW,
        LeftHandSideExpression, MEMBER_EXPR_MEMBER, MEMBER_EXPR_NEW, MEMBER_EXPR_PRIMARY,
        MEMBER_EXPR_PRIVATE_PROPERTY, MEMBER_EXPR_PROPERTY, MemberExpression, NEW_EXPR_MEMBER,
        NEW_EXPR_NEW, NewExpression,
    },
    operations::{IteratorKind, get_iterator, is_constructor},
    semantics::{EvaluateExpressionTag, general_evaluate, identifier, primary},
    types::completion_record::{CRKAbrupt, CRKNormal, CompletionRecord, CompletionRecordError},
    values::{
        ReferenceOrValue, Value,
        reference::{Reference, ReferenceBase, ReferenceName, get_value},
    },
};

pub fn evaluate(lhs: LeftHandSideExpression) -> ReferenceOrValue {
    match lhs.tag {
        LEFT_HAND_SIDE_EXPR_NEW => {
            let new_data = unsafe { *lhs.data.new };

            return evaluate_new_expr(&new_data);
        }
        LEFT_HAND_SIDE_EXPR_CALL => {
            let call_data = unsafe { *lhs.data.call };

            return evaluate_call(&call_data);
        }
        _ => unreachable!("Unknown left-hand side expression tag: {}", lhs.tag),
    }
}

pub fn evaluate_new_expr(new_expr: &NewExpression) -> ReferenceOrValue {
    match new_expr.tag {
        NEW_EXPR_MEMBER => {
            let member_data = unsafe { *new_expr.data.member };
            return evaluate_member(&member_data);
        }
        NEW_EXPR_NEW => {
            let new_data = unsafe { *new_expr.data.new };
            let res = evaluate_new(NewOrMember::New(new_data), None);

            return ReferenceOrValue::Value(res.unwrap().value);
        }
        _ => unreachable!("Unknown new expression tag: {}", new_expr.tag),
    }
}

pub fn evaluate_call(call: &CallExpression) -> ReferenceOrValue {
    match call.tag {
        CALL_EXPR_MEMBER => {
            let object_data = unsafe { *call.data.member.object };
            let expression_data = unsafe { *call.data.member.expr };

            let base_reference = evaluate_call(&object_data);
            let base_value = get_value(base_reference);

            let strict = true;

            let res = evaluate_property_access_with_expression_key(
                base_value.unwrap().value,
                EvaluateExpressionTag::Expression(expression_data),
                strict,
            );

            return ReferenceOrValue::Reference(res.unwrap().value);
        }
        CALL_EXPR_PROPERTY => {
            let object_data = unsafe { *call.data.property.object };
            let property_name_data = unsafe { *call.data.property.property };

            let base_reference = evaluate_call(&object_data);
            let base_value = get_value(base_reference);

            let strict = true;

            let res = evaluate_property_access_with_identifier_key(
                base_value.unwrap().value,
                property_name_data,
                strict,
            );

            return ReferenceOrValue::Reference(res);
        }
        CALL_EXPR_PRIVATE_PROPERTY => {
            todo!("Private property access evaluation in call expression")
        }
        _ => unreachable!("Unknown call expression tag: {}", call.tag),
    }
}

pub fn evaluate_member(member: &MemberExpression) -> ReferenceOrValue {
    match member.tag {
        MEMBER_EXPR_PRIMARY => {
            let primary_data = unsafe { *member.data.primary };
            return primary::evaluate(primary_data);
        }
        MEMBER_EXPR_MEMBER => {
            let object_data = unsafe { *member.data.member.object };
            let expression_data = unsafe { *member.data.member.expr };

            let base_reference = evaluate_member(&object_data);
            let base_value = get_value(base_reference);

            // NOTE: Uhhhhh
            let strict = true;

            let res = evaluate_property_access_with_expression_key(
                base_value.unwrap().value,
                EvaluateExpressionTag::Expression(expression_data),
                strict,
            )
            .unwrap();

            return ReferenceOrValue::Reference(res.value);
        }
        MEMBER_EXPR_PROPERTY => {
            let object_data = unsafe { *member.data.property.object };
            let property_name_data = unsafe { *member.data.property.property };

            let base_reference = evaluate_member(&object_data);
            let base_value = get_value(base_reference);

            let strict = true;

            let res = evaluate_property_access_with_identifier_key(
                base_value.unwrap().value,
                property_name_data,
                strict,
            );

            return ReferenceOrValue::Reference(res);
        }
        MEMBER_EXPR_PRIVATE_PROPERTY => {
            todo!("Private property access evaluation")
        }
        MEMBER_EXPR_NEW => {
            let member_data = unsafe { *member.data.new.callee };
            let args = unsafe { *member.data.new.arguments };

            let res = evaluate_new(NewOrMember::Member(member_data), Some(args));

            return ReferenceOrValue::Value(res.unwrap().value);
        }
        _ => unreachable!("Unknown member expression tag: {}", member.tag),
    }
}

pub fn evaluate_property_access_with_expression_key(
    base_value: Value,
    expression: EvaluateExpressionTag,
    strict: bool,
) -> Result<CompletionRecord<Reference>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
    let property_name_reference = general_evaluate(expression);
    let maybe_property_name_value = get_value(property_name_reference);

    if let Err(e) = maybe_property_name_value {
        return Err(CompletionRecord {
            kind: CRKAbrupt::Throw,
            value: e.value,
            target: None,
        });
    }

    let property_name_value = maybe_property_name_value.unwrap().unwrapped().clone();

    Ok(CompletionRecord {
        kind: CRKNormal,
        value: Reference {
            base: ReferenceBase::Value(base_value),
            referenced_name: ReferenceName::Value(property_name_value),
            strict,
            this_value: None,
        },
        target: None,
    })
}

pub fn evaluate_property_access_with_identifier_key(
    base_value: Value,
    identifier_name: IdentifierNameTokenData,
    strict: bool,
) -> Reference {
    let property_name_string = identifier::string_value(identifier_name);

    Reference {
        base: ReferenceBase::Value(base_value),
        referenced_name: ReferenceName::Value(Value::String(property_name_string)),
        strict,
        this_value: None,
    }
}

pub enum NewOrMember {
    New(NewExpression),
    Member(MemberExpression),
}

pub fn evaluate_new(
    construct_expr: NewOrMember,
    arguments: Option<Arguments>,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
    let reference = match construct_expr {
        NewOrMember::New(new) => {
            let eval_expr = EvaluateExpressionTag::NewExpression(new);
            general_evaluate(eval_expr)
        }
        NewOrMember::Member(member) => {
            let eval_expr = EvaluateExpressionTag::MemberExpression(member);
            general_evaluate(eval_expr)
        }
    };

    let constructor = get_value(reference)?.value;

    let _args_list = if arguments.is_none() {
        Vec::<Value>::new()
    } else {
        todo!()
    };

    if !is_constructor(&constructor) {
        return Err(CompletionRecord {
            kind: CRKAbrupt::Throw,
            value: CompletionRecordError::TypeError,
            target: None,
        });
    }

    todo!("Construct")
}

pub fn argument_list_evaluation(
    arguments: Arguments,
) -> Result<CompletionRecord<Vec<Value>>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
    let mut args_list = Vec::<Value>::new();
    let seq = collect_seq(arguments.arguments);

    let is_spread_elems =
        unsafe { std::slice::from_raw_parts(arguments.is_spread, arguments.arguments.len) }
            .iter()
            .copied()
            .collect::<Vec<bool>>();

    for (i, arg) in seq.iter().enumerate() {
        if is_spread_elems[i] {
            let mut list = Vec::<Value>::new();
            let spread_ref = general_evaluate(EvaluateExpressionTag::AssignmentExpression(*arg));
            let spread_obj = get_value(spread_ref)?.value;

            // let iterator_rec = get_iterator(&spread_obj, IteratorKind::Sync);
        }
    }

    todo!()
}
