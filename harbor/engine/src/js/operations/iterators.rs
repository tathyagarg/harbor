use crate::js::{
    operations::{call, get, get_method, to_boolean},
    types::completion_record::{
        CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
    },
    values::{
        Value,
        object::{Object, PropertyKey},
        symbol::SYMBOL_ITERATOR,
    },
};

pub enum IteratorKind {
    Sync,
    Async,
}

#[derive(Debug, Clone)]
pub struct Iterator {
    pub iterator: Object,
    pub next_method: Value,
    pub done: bool,
}

pub fn get_iterator_direct(
    obj: &Object,
) -> Result<CompletionRecord<Iterator>, CompletionRecord<CompletionRecordError, CRKThrow>> {
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
) -> Result<CompletionRecord<Iterator>, CompletionRecord<CompletionRecordError, CRKThrow>> {
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
) -> Result<CompletionRecord<Iterator>, CompletionRecord<CompletionRecordError, CRKThrow>> {
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
    iterator: &mut Iterator,
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
    iterator: &mut Iterator,
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
    iterator: &mut Iterator,
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

pub fn create_list_iterator_record(_list: Vec<Value>) -> Iterator {
    todo!("No infra to do allat")
}
