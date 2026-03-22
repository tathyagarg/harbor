use std::{cell::RefCell, rc::Rc};

use crate::js::{
    types::completion_record::{
        CompletionRecord, CompletionRecordError, CompletionRecordNormal, CompletionRecordThrow,
    },
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

fn ordinary_define_own_property(
    object: &mut Object,
    key: PropertyKey,
    desc: PropertyDescriptor,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>> {
    let _current = ordinary_get_own_property(object, key.clone());
    if let Some(current) = _current {
        let extensible = ordinary_is_extensible(object);

        Ok(CompletionRecordNormal(
            validate_and_apply_property_descriptor(
                Some(object),
                key,
                extensible,
                &desc,
                Some(&current),
            ),
        ))
    } else {
        Err(CompletionRecordThrow(CompletionRecordError::Misc(format!(
            "Property {:?} does not exist on object",
            key
        ))))
    }
}

fn is_compatible_property_descriptor(
    extensible: bool,
    desc: &PropertyDescriptor,
    current: Option<&PropertyDescriptor>,
) -> bool {
    validate_and_apply_property_descriptor(None, PropertyKey::empty(), extensible, desc, current)
}

fn validate_and_apply_property_descriptor(
    object: Option<&mut Object>,
    key: PropertyKey,
    extensible: bool,
    desc: &PropertyDescriptor,
    current: Option<&PropertyDescriptor>,
) -> bool {
    if current.is_none() {
        if !extensible {
            return false;
        }

        if object.is_none() {
            return true;
        }

        let object = object.unwrap();
        match object {
            Object::Ordinary(ordinary) => {
                ordinary.properties.insert(key, desc.clone());
            }
            _ => panic!(),
        }

        return true;
    }

    let current = current.unwrap();
    if !current.configurable() {
        if desc.field("configurable").is_some() && desc.configurable() {
            return false;
        }

        if desc.field("enumerable").is_some() && (desc.enumerable() != current.enumerable()) {
            return false;
        }

        if !desc.is_generic_descriptor()
            && (desc.is_accessor_descriptor() != current.is_accessor_descriptor())
        {
            return false;
        }

        if current.is_accessor_descriptor() {
            if (desc.field("get").is_some()
                && !same_value(
                    &desc.field("get").unwrap_or(Value::Undefined),
                    &current.field("get").unwrap_or(Value::Undefined),
                ))
                || (desc.field("set").is_some()
                    && !same_value(
                        &desc.field("set").unwrap_or(Value::Undefined),
                        &current.field("set").unwrap_or(Value::Undefined),
                    ))
            {
                return false;
            }
        } else if same_value(
            &current.field("writable").unwrap_or(Value::Undefined),
            &Value::Boolean(false),
        ) {
            if desc.field("writable").is_some()
                && same_value(
                    &desc.field("writable").unwrap_or(Value::Undefined),
                    &Value::Boolean(true),
                )
            {
                return false;
            }

            if desc.field("value").is_some() {
                return same_value(
                    &desc.field("value").unwrap_or(Value::Undefined),
                    &current.field("value").unwrap_or(Value::Undefined),
                );
            }
        }
    }

    if object.is_some() {
        if current.is_data_descriptor() && desc.is_accessor_descriptor() {
            let configurable = (if desc.field("configurable").is_some() {
                desc.field("configurable").unwrap_or(Value::Undefined)
            } else {
                Value::Boolean(current.configurable())
            })
            .unwrap_bool()
            .unwrap();

            let enumerable = (if desc.field("enumerable").is_some() {
                desc.field("enumerable").unwrap_or(Value::Undefined)
            } else {
                Value::Boolean(current.enumerable())
            })
            .unwrap_bool()
            .unwrap();

            match object.unwrap() {
                Object::Ordinary(ordinary) => {
                    ordinary.properties.insert(
                        key,
                        PropertyDescriptor::Accessor {
                            get: desc.field("get").unwrap_or(Value::Undefined),
                            set: desc.field("set").unwrap_or(Value::Undefined),
                            enumerable,
                            configurable,
                        },
                    );
                }
                _ => panic!(),
            }
        } else if current.is_accessor_descriptor() && desc.is_data_descriptor() {
            let configurable = (if desc.field("configurable").is_some() {
                desc.field("configurable").unwrap_or(Value::Undefined)
            } else {
                Value::Boolean(current.configurable())
            })
            .unwrap_bool()
            .unwrap();

            let enumerable = (if desc.field("enumerable").is_some() {
                desc.field("enumerable").unwrap_or(Value::Undefined)
            } else {
                Value::Boolean(current.enumerable())
            })
            .unwrap_bool()
            .unwrap();

            match object.unwrap() {
                Object::Ordinary(ordinary) => {
                    ordinary.properties.insert(
                        key,
                        PropertyDescriptor::Data {
                            value: desc.field("value").unwrap_or(Value::Undefined),
                            writable: desc
                                .field("writable")
                                .unwrap_or(Value::Boolean(false))
                                .unwrap_bool()
                                .unwrap(),
                            enumerable,
                            configurable,
                        },
                    );
                }
                _ => panic!(),
            }
        } else {
            let ordinary = match object.unwrap() {
                Object::Ordinary(ordinary) => ordinary,
                _ => panic!(),
            };

            for field in desc.fields() {
                ordinary
                    .properties
                    .insert(PropertyKey::from(field), desc.clone());
            }
        }
    }

    true
}

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
