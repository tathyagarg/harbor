use crate::js::{
    operations::get_method,
    types::completion_record::{CRKThrow, CompletionRecord, CompletionRecordError},
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

pub fn get_iterator_from_method(
    obj: &Value,
    method: &Object,
) -> Result<CompletionRecord<Iterator>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    todo!()
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
    let result = if value.is_none() { todo!() } else { todo!() };
}

pub fn iterator_step(
    iterator: &mut Iterator,
) -> Result<CompletionRecord<Option<Object>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    todo!()
}

pub fn iterator_step_value(
    iterator: &mut Iterator,
) -> Result<CompletionRecord<Option<Value>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    todo!()
}
