use std::collections::HashMap;

use crate::js::{
    SLOT_EXTENSIBLE, SLOT_PRIVATE_ELEMENTS,
    operations::{is_callable, to_object},
    types::completion_record::{
        CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
        CompletionRecordThrow, UNUSED,
    },
    values::{
        Value,
        object::{MiscObject, Object, ObjectTrait, PropertyDescriptor, PropertyKey, SlotValue},
    },
};

pub fn make_basic_object(internal_slots_list: Vec<String>) -> Object {
    let internal_slots = [
        &internal_slots_list[..],
        &[String::from(SLOT_PRIVATE_ELEMENTS)][..],
    ]
    .concat();

    let mut object = MiscObject {
        internal_slots: internal_slots
            .iter()
            .map(|name| (name.clone(), SlotValue::Undefined))
            .collect(),
        properties: HashMap::new(),
        // with an empty method proxy the misc object gets default ordinary object behavior
        method_proxy: None,
    };

    object.internal_slots.insert(
        SLOT_PRIVATE_ELEMENTS.to_string(),
        SlotValue::List(Vec::new()),
    );

    // NOTE: Step 5 is skipped because setting method_proxy to None gives that behavior by default

    if internal_slots.contains(&String::from(SLOT_EXTENSIBLE)) {
        object.internal_slots.insert(
            SLOT_EXTENSIBLE.to_string(),
            SlotValue::Value(Value::Boolean(true)),
        );
    }

    return Object::Misc(object);
}

pub fn get(
    obj: &Object,
    key: &PropertyKey,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let res = obj.get(key, &Value::Object(obj.clone()));

    return Ok(CompletionRecordNormal(res.unwrap_or(Value::Undefined)));
}

pub fn getv(
    value: &Value,
    property_key: &PropertyKey,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let o = to_object(value)?.value;
    let res = o.get(property_key, value);

    return Ok(CompletionRecordNormal(res.unwrap_or(Value::Undefined)));
}

pub fn set(
    obj: &mut Object,
    key: &PropertyKey,
    value: &Value,
    throw: bool,
) -> Result<CompletionRecord<UNUSED>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let mut obj_value = Value::Object(obj.clone());
    let success = obj.set(key, value, &mut obj_value);
    if let Value::Object(o) = obj_value {
        *obj = o;
    }

    if !success && throw {
        return Err(CompletionRecordThrow(CompletionRecordError::TypeError));
    }

    return Ok(CompletionRecordNormal(()));
}

pub fn create_data_property(
    obj: &mut Object,
    key: &PropertyKey,
    value: &Value,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let new_desc = PropertyDescriptor::Data {
        value: value.clone(),
        writable: true,
        enumerable: true,
        configurable: true,
    };

    return Ok(CompletionRecordNormal(
        obj.define_own_property(key, new_desc),
    ));
}

pub fn get_method(
    value: &Value,
    key: &PropertyKey,
) -> Result<CompletionRecord<Option<Object>>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let func = getv(value, key)?.value;

    if matches!(func, Value::Undefined | Value::Null) {
        return Ok(CompletionRecordNormal(None));
    }

    if !is_callable(&func) {
        return Err(CompletionRecordThrow(CompletionRecordError::TypeError));
    }

    let func_obj = func.unwrap_object();

    Ok(CompletionRecordNormal(func_obj))
}

pub fn has_own_property(
    obj: &Object,
    key: &PropertyKey,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let desc = obj.get_own_property(key);
    if desc.is_none() {
        return Ok(CompletionRecordNormal(false));
    }

    return Ok(CompletionRecordNormal(true));
}

pub fn call(
    func: &Value,
    target: &Value,
    args: Vec<Value>,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    if let Value::Object(func_obj) = func {
        return Ok(CompletionRecordNormal(func_obj.call(target, args)));
    }

    return Err(CompletionRecordThrow(CompletionRecordError::TypeError));
}
