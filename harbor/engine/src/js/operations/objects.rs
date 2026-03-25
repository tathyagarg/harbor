use std::collections::HashMap;

use crate::js::{
    types::completion_record::{
        CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
        CompletionRecordThrow, UNUSED,
    },
    values::{
        Value,
        object::{MiscObject, Object, ObjectTrait, PropertyDescriptor, PropertyKey},
    },
};

pub fn make_basic_object(internal_slots_list: Vec<String>) -> Object {
    let internal_slots = [
        &internal_slots_list[..],
        &[String::from("private_elements")][..],
    ]
    .concat();

    let mut object = MiscObject {
        properties: internal_slots
            .iter()
            .map(|slot| {
                (
                    PropertyKey::from(slot.clone()),
                    PropertyDescriptor::Data {
                        value: Value::Undefined,
                        writable: false,
                        enumerable: false,
                        configurable: false,
                    },
                )
            })
            .collect::<HashMap<PropertyKey, PropertyDescriptor>>(),
    };

    object.properties.insert(
        PropertyKey::from("private_elements"),
        PropertyDescriptor::Data {
            value: Value::Undefined,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    );

    todo!()
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
