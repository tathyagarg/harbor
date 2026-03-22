use std::{cell::RefCell, rc::Rc};

use crate::js::{
    types::completion_record::{CompletionRecord, CompletionRecordError},
    values::{
        Value,
        object::{ArrayObject, Object, OrdinaryObject, PropertyDescriptor, PropertyKey},
        same_value,
    },
};

// 10.4
pub mod exotics;

fn ordinary_get_prototype_of(object: &Object) -> Rc<RefCell<Option<Object>>> {
    match object {
        Object::Ordinary(OrdinaryObject { prototype, .. }) => prototype.clone(),
        _ => Rc::new(RefCell::new(None)),
    }
}

fn ordinary_set_prototype_of(object: &mut Object, prototype: Option<Object>) -> bool {
    let current = ordinary_get_prototype_of(object);

    if same_value(
        &prototype
            .as_ref()
            .map(|o| Value::Object(o.clone()))
            .unwrap_or(Value::Null),
        &current
            .borrow()
            .as_ref()
            .map(|o| Value::Object(o.clone()))
            .unwrap_or(Value::Null),
    ) {
        return true;
    }

    let extensible = match object {
        Object::Ordinary(OrdinaryObject { extensible, .. }) => extensible,
        Object::Array(ArrayObject { extensible, .. }) => extensible,
    }
    .clone();

    if !extensible {
        return false;
    }

    let mut p = prototype.clone();
    let mut done = false;

    while !done {
        if p.is_none() {
            done = true;
        } else if same_value(
            &Value::Object(p.as_ref().unwrap().clone()),
            &Value::Object(object.clone()),
        ) {
            return false;
        } else {
            p = ordinary_get_prototype_of(&p.unwrap())
                .borrow()
                .as_ref()
                .map(|o| o.clone());
        }
    }

    match object {
        Object::Ordinary(OrdinaryObject {
            prototype: old_proto,
            ..
        }) => {
            old_proto.borrow_mut().replace(prototype.unwrap());
        }
        _ => panic!(),
    }

    true
}

pub fn ordinary_is_extensible(object: &Object) -> bool {
    match object {
        Object::Ordinary(OrdinaryObject { extensible, .. }) => *extensible,
        Object::Array(ArrayObject { extensible, .. }) => *extensible,
    }
}

pub fn ordinary_prevent_extensions(object: &mut Object) -> bool {
    match object {
        Object::Ordinary(OrdinaryObject { extensible, .. }) => {
            *extensible = false;
            true
        }
        Object::Array(ArrayObject { extensible, .. }) => {
            *extensible = false;
            true
        }
    }
}

pub fn ordinary_get_own_property(object: &Object, key: PropertyKey) -> Option<PropertyDescriptor> {
    let ordinary = match object {
        Object::Ordinary(ordinary) => ordinary,
        _ => return None,
    };

    if !ordinary.properties.contains_key(&key) {
        return None;
    }

    let x = ordinary.properties.get(&key).unwrap();

    let desc = if let PropertyDescriptor::Data {
        value,
        writable,
        enumerable,
        configurable,
    } = x
    {
        PropertyDescriptor::Data {
            value: value.clone(),
            writable: *writable,
            enumerable: *enumerable,
            configurable: *configurable,
        }
    } else if let PropertyDescriptor::Accessor {
        get,
        set,
        enumerable,
        configurable,
    } = x
    {
        PropertyDescriptor::Accessor {
            get: get.clone(),
            set: set.clone(),
            enumerable: *enumerable,
            configurable: *configurable,
        }
    } else {
        panic!()
    };

    Some(desc)
}

// fn ordinary_define_own_property(
//     object: &mut Object,
//     key: PropertyKey,
//     desc: PropertyDescriptor,
// ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>> {
// }
//
// fn is_compatible_property_descriptor(
//     extensible: bool,
//     desc: &PropertyDescriptor,
//     current: Option<&PropertyDescriptor>,
// ) -> bool {
// }
//
// fn validate_and_apply_property_descriptor(
//     object: Option<&mut Object>,
//     key: PropertyKey,
//     extensible: bool,
//     desc: &PropertyDescriptor,
//     current: Option<&PropertyDescriptor>,
// ) -> bool {
// }
//
// fn ordinary_has_property(
//     object: &Object,
//     key: PropertyKey,
// ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>> {
// }
//
// fn ordinary_get(
//     object: &Object,
//     key: PropertyKey,
//     receiver: Value,
// ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError>> {
// }
//
// fn ordinary_set(
//     object: &mut Object,
//     key: PropertyKey,
//     value: Value,
//     receiver: Value,
// ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>> {
// }
//
// fn ordinary_set_with_own_descriptor(
//     object: &mut Object,
//     key: PropertyKey,
//     value: Value,
//     receiver: Value,
//     own_desc: Option<PropertyDescriptor>,
// ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>> {
// }
//
// fn ordinary_delete(
//     object: &mut Object,
//     key: PropertyKey,
// ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>> {
// }
//
// fn ordinary_own_property_keys(object: &Object) -> Vec<PropertyKey>;
