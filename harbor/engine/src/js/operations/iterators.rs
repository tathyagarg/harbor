use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::LazyLock};

use crate::js::{
    r#abstract::{ITEREATOR_PROTOTYPE, create_iterator_from_closure},
    behaviours::{builtin_functions::BuiltinFunction, ordinary_object_create},
    executable::{context::resolve_binding, environment::EnvironmentRecord},
    operations::{call, create_data_property_or_throw, get, get_method, to_boolean},
    semantics::{
        evaluate::expressions::{EvaluateExpressionTag, expression_evaluate},
        r#static::{ParseNode, string_value},
    },
    types::completion_record::{
        CRKAbrupt, CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
    },
    values::{
        ReferenceOrValue, Value,
        object::{Object, ObjectTrait, OrdinaryObject, PropertyKey},
        reference::{initialize_referenced_binding, put_value},
        symbol::SYMBOL_ITERATOR,
    },
};

pub enum IteratorKind {
    Sync,
    Async,
}

#[derive(Debug, Clone)]
pub struct Iterator<T: ObjectTrait + Clone + Debug> {
    pub iterator: T,
    pub next_method: Value,
    pub done: bool,
}

pub fn get_iterator_direct(
    obj: &Object,
) -> Result<CompletionRecord<Iterator<Object>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let next_method = get(obj, &PropertyKey::from("next"))?.value;
    let iterator_record = Iterator {
        iterator: obj.clone(),
        next_method: next_method,
        done: false,
    };

    return Ok(CompletionRecordNormal(iterator_record));
}

pub fn get_iterator_from_method(
    obj: &Value,
    method: &Object,
) -> Result<CompletionRecord<Iterator<Object>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let iterator = call(&Value::Object(method.clone()), obj, Vec::new())?.value;

    if let Value::Object(iterator_obj) = iterator {
        return get_iterator_direct(&iterator_obj);
    } else {
        return Err(CompletionRecord {
            kind: CRKThrow,
            value: CompletionRecordError::TypeError,
            target: None,
        });
    }
}

pub fn get_iterator(
    obj: &Value,
    kind: IteratorKind,
) -> Result<CompletionRecord<Iterator<Object>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let method = match kind {
        IteratorKind::Sync => get_method(obj, &PropertyKey::Symbol(SYMBOL_ITERATOR.clone())),
        IteratorKind::Async => {
            todo!()
        }
    }?
    .value;

    if method.is_none() {
        return Err(CompletionRecord {
            kind: CRKThrow,
            value: CompletionRecordError::TypeError,
            target: None,
        });
    }

    return get_iterator_from_method(obj, &method.unwrap());
}

pub fn iterator_next(
    iterator: &mut Iterator<Object>,
    value: Option<Value>,
) -> Result<CompletionRecord<Object>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let result = if value.is_none() {
        call(
            &iterator.next_method,
            &Value::Object(iterator.iterator.clone()),
            Vec::new(),
        )
    } else {
        call(
            &iterator.next_method,
            &Value::Object(iterator.iterator.clone()),
            vec![value.unwrap()],
        )
    };

    if result.is_err() {
        iterator.done = true;
        return Err(result.err().unwrap());
    }

    let result_value = result.unwrap().value;

    if let Value::Object(result_obj) = result_value {
        return Ok(CompletionRecordNormal(result_obj));
    } else {
        iterator.done = true;
        return Err(CompletionRecord {
            kind: CRKThrow,
            value: CompletionRecordError::TypeError,
            target: None,
        });
    }
}

pub fn iterator_complete(
    iterator_result: &Object,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let res = get(iterator_result, &PropertyKey::from("done"))?.value;

    Ok(CompletionRecordNormal(to_boolean(&res)))
}

pub fn iterator_value(
    iterator_result: &Object,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let res = get(iterator_result, &PropertyKey::from("value"))?.value;

    Ok(CompletionRecordNormal(res))
}

pub fn iterator_step(
    iterator: &mut Iterator<Object>,
) -> Result<CompletionRecord<Option<Object>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let result = iterator_next(iterator, None)?.value;
    let done = iterator_complete(&result);

    if done.is_err() {
        iterator.done = true;
        return Err(done.err().unwrap());
    }

    let done_value = done.unwrap().value;
    if done_value {
        iterator.done = true;
        return Ok(CompletionRecordNormal(None));
    }

    return Ok(CompletionRecordNormal(Some(result)));
}

pub fn iterator_step_value(
    iterator: &mut Iterator<Object>,
) -> Result<CompletionRecord<Option<Value>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let result = iterator_step(iterator)?.value;

    if result.is_none() {
        return Ok(CompletionRecordNormal(None));
    }

    let value = iterator_value(&result.unwrap());
    if value.is_err() {
        iterator.done = true;
    }

    return value.map(|v| CompletionRecordNormal(Some(v.value)));
}

pub fn create_iterator_result_object(value: Value, done: bool) -> Object {
    let mut obj = Object::Ordinary(ordinary_object_create(
        Some(Object::Ordinary(OrdinaryObject::prototype())),
        vec![],
    ));

    create_data_property_or_throw(&mut obj, &PropertyKey::from("value"), &value).unwrap();
    create_data_property_or_throw(&mut obj, &PropertyKey::from("done"), &Value::Boolean(done))
        .unwrap();

    return obj;
}

pub const GENERATOR_NEXT: LazyLock<BuiltinFunction> = LazyLock::new(|| BuiltinFunction {
    prototype: Rc::new(RefCell::new(Some(Object::Ordinary(
        OrdinaryObject::prototype(),
    )))),
    extensible: true,
    realm: None,
    initial_name: "Generator.prototype.next".to_string(),
    is_async: false,
    internal_closure: |_this, _args| Value::Undefined,
});

pub fn create_list_iterator_record(list: Vec<Value>) -> Iterator<Object> {
    let closure: Box<(dyn Fn() -> Option<Value>)> = Box::new(move || -> Option<Value> {
        for _item in &list {
            todo!("GeneratorYield(CreateIteratorResultObject(E, false))");
        }

        return None;
    });

    let iterator = create_iterator_from_closure(closure, None, ITEREATOR_PROTOTYPE.clone());

    return Iterator {
        iterator: Object::Generator(iterator),
        next_method: Value::Object(Object::BuiltinFunction(GENERATOR_NEXT.clone())),
        done: false,
    };
}

pub fn iterator_binding_initialization(
    formals: &ParseNode,
    iterator_record: &mut Iterator<Object>,
    environment: Option<Rc<RefCell<EnvironmentRecord>>>,
) -> Result<CompletionRecord, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
    if let ParseNode::FormalParameters(formals) = formals {
        for param in formals.iter() {
            let name = unsafe { *param.name };
            let raw_initializer = unsafe { *param.initializer };

            let initializer = if raw_initializer.has_value {
                Some(unsafe { raw_initializer.value.value })
            } else {
                None
            };

            let binding_id = string_value(name);
            let mut lhs = resolve_binding(binding_id.clone(), environment.clone())
                .unwrap()
                .value;

            println!(
                "Initializing binding {:?}\nRecord: {:#?}",
                binding_id, iterator_record
            );

            let val = if !iterator_record.done {
                let next = iterator_step_value(iterator_record).unwrap().value;
                if next.is_some() {
                    next.unwrap()
                } else {
                    Value::Undefined
                }
            } else if initializer.is_some() {
                let default = expression_evaluate(&EvaluateExpressionTag::AssignmentExpression(
                    initializer.unwrap(),
                ));

                default.get_value().unwrap().value
            } else {
                Value::Undefined
            };

            if environment.is_none() {
                let mut lhs_ref = ReferenceOrValue::Reference(lhs.clone());
                put_value(&mut lhs_ref, &val).unwrap();

                lhs = if let ReferenceOrValue::Reference(r) = lhs_ref {
                    r
                } else {
                    unreachable!()
                };
            }

            initialize_referenced_binding(&mut lhs, &val).unwrap();
        }
    }

    return Ok(CompletionRecordNormal(()));
}
