use crate::js::{
    collect_seq,
    expr::{
        Arguments, CALL_EXPR_COVER, CALL_EXPR_MEMBER, CALL_EXPR_PRIVATE_PROPERTY,
        CALL_EXPR_PROPERTY, CallExpression, IdentifierNameTokenData, LEFT_HAND_SIDE_EXPR_CALL,
        LEFT_HAND_SIDE_EXPR_NEW, LeftHandSideExpression, MEMBER_EXPR_MEMBER, MEMBER_EXPR_NEW,
        MEMBER_EXPR_PRIMARY, MEMBER_EXPR_PRIVATE_PROPERTY, MEMBER_EXPR_PROPERTY, MemberExpression,
        NEW_EXPR_MEMBER, NEW_EXPR_NEW, NewExpression,
    },
    operations::{
        IteratorKind, call, get_iterator, is_callable, is_constructor, iterator_step_value,
    },
    semantics::{
        evaluate::expressions::{EvaluateExpressionTag, expression_evaluate, primary},
        r#static::string_value,
    },
    types::completion_record::{
        CRKAbrupt, CRKNormal, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
    },
    values::{
        ReferenceOrValue, Value,
        reference::{Reference, ReferenceBase, ReferenceName, get_value},
    },
};

pub fn evaluate(lhs: &LeftHandSideExpression) -> ReferenceOrValue {
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
            let base_value = get_value(&base_reference);

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
            let base_value = get_value(&base_reference);

            let strict = true;

            let res = evaluate_property_access_with_identifier_key(
                base_value.unwrap().value,
                property_name_data,
                strict,
            );

            return ReferenceOrValue::Reference(res);
        }
        CALL_EXPR_COVER => {
            let expr = unsafe { *call.data.cover.callee };
            let arguments = unsafe { *call.data.cover.arguments };

            let reference = expression_evaluate(&EvaluateExpressionTag::MemberExpression(expr));
            let func = reference.get_value().unwrap().value;

            ReferenceOrValue::Value(_evaluate_call(func, &reference, arguments).unwrap().value)
        }
        CALL_EXPR_PRIVATE_PROPERTY => {
            todo!("Private property access evaluation in call expression")
        }
        _ => unreachable!("Unknown call expression tag: {}", call.tag),
    }
}

fn _evaluate_call(
    func: Value,
    reference: &ReferenceOrValue,
    arguments: Arguments,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
    let this_value = if let ReferenceOrValue::Reference(func_ref) = reference {
        if func_ref.is_property_reference() {
            func_ref.get_this_value()
        } else {
            let ref_env = &func_ref.base;
            if let ReferenceBase::EnvironmentRecord(env) = ref_env {
                env.borrow()
                    .with_base_object()
                    .map(|obj| Value::Object(obj))
                    .unwrap_or(Value::Undefined)
            } else {
                unreachable!()
            }
        }
    } else {
        Value::Undefined
    };

    let arg_list = evaluate_arguments(&arguments);

    if !func.is_object() {
        return Err(CompletionRecord {
            kind: CRKAbrupt::Throw,
            value: CompletionRecordError::TypeError,
            target: None,
        });
    }

    // NOTE: Safety last
    // if !is_callable(&func) {
    //     return Err(CompletionRecord {
    //         kind: CRKAbrupt::Throw,
    //         value: CompletionRecordError::TypeError,
    //         target: None,
    //     });
    // }

    let res = call(&func, &this_value, arg_list);

    res.map_err(|e| CompletionRecord {
        kind: CRKAbrupt::Throw,
        value: e.value,
        target: None,
    })
}

pub fn evaluate_arguments(arguments: &Arguments) -> Vec<Value> {
    let mut args_list = Vec::<Value>::new();
    let seq = collect_seq(&arguments.arguments);
    let spread =
        unsafe { std::slice::from_raw_parts(arguments.is_spread, arguments.arguments.len) }
            .iter()
            .copied()
            .collect::<Vec<bool>>();

    for (is_spread, arg) in spread.iter().zip(seq.iter()) {
        if *is_spread {
            let spread_ref =
                expression_evaluate(&EvaluateExpressionTag::AssignmentExpression(*arg));
            let spread_obj = get_value(&spread_ref).unwrap().value;

            let mut iterator_rec = get_iterator(&spread_obj, IteratorKind::Sync).unwrap().value;

            loop {
                let next = iterator_step_value(&mut iterator_rec).unwrap().value;
                if let Some(value) = next {
                    args_list.push(value);
                } else {
                    break;
                }
            }
        } else {
            let arg_ref = expression_evaluate(&EvaluateExpressionTag::AssignmentExpression(*arg));
            let arg_value = get_value(&arg_ref).unwrap().value;

            args_list.push(arg_value);
        }
    }

    args_list
}

pub fn evaluate_member(member: &MemberExpression) -> ReferenceOrValue {
    match member.tag {
        MEMBER_EXPR_PRIMARY => {
            let primary_data = unsafe { *member.data.primary };
            return primary::evaluate(&primary_data);
        }
        MEMBER_EXPR_MEMBER => {
            let object_data = unsafe { *member.data.member.object };
            let expression_data = unsafe { *member.data.member.expr };

            let base_reference = evaluate_member(&object_data);
            let base_value = get_value(&base_reference);

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
            let base_value = get_value(&base_reference);

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
    let property_name_reference = expression_evaluate(&expression);
    let maybe_property_name_value = get_value(&property_name_reference);

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
    let property_name_string = string_value(identifier_name);

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
            expression_evaluate(&eval_expr)
        }
        NewOrMember::Member(member) => {
            let eval_expr = EvaluateExpressionTag::MemberExpression(member);
            expression_evaluate(&eval_expr)
        }
    };

    let constructor = get_value(&reference)?.value;

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
    let seq = collect_seq(&arguments.arguments);

    let is_spread_elems =
        unsafe { std::slice::from_raw_parts(arguments.is_spread, arguments.arguments.len) }
            .iter()
            .copied()
            .collect::<Vec<bool>>();

    for (i, arg) in seq.iter().enumerate() {
        if is_spread_elems[i] {
            let spread_ref =
                expression_evaluate(&EvaluateExpressionTag::AssignmentExpression(*arg));
            let spread_obj = get_value(&spread_ref)?.value;

            let mut iterator_rec = get_iterator(&spread_obj, IteratorKind::Sync).unwrap().value;

            loop {
                let next = iterator_step_value(&mut iterator_rec).unwrap().value;
                if let Some(value) = next {
                    args_list.push(value);
                } else {
                    break;
                }
            }
        } else {
            let arg_ref = expression_evaluate(&EvaluateExpressionTag::AssignmentExpression(*arg));
            let arg_value = get_value(&arg_ref)?.value;

            args_list.push(arg_value);
        }
    }

    Ok(CompletionRecordNormal(args_list))
}
