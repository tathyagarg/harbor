use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::js::{
    operations::create_data_property,
    types::completion_record::{
        CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
        CompletionRecordThrow,
    },
    values::{
        Value,
        object::{
            ArrayObject, Object, ObjectTrait, OrdinaryObject, PropertyDescriptor, PropertyKey,
        },
        same_value,
    },
};

// 10.4
pub mod exotics;

pub fn ordinary_object_create(
    prototype: Option<Object>,
    additional_internal_slots_list: Vec<String>,
) -> Object {
    // let obj = make_basic_object();

    todo!()
}

pub fn ordinary_get_prototype_of(object: &Object) -> Rc<RefCell<Option<Object>>> {
    match object {
        Object::Ordinary(OrdinaryObject { prototype, .. }) => prototype.clone(),
        _ => Rc::new(RefCell::new(None)),
    }
}

pub fn ordinary_set_prototype_of(object: &mut Object, prototype: Option<Object>) -> bool {
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
        Object::Ordinary(OrdinaryObject { extensible, .. }) => *extensible,
        Object::Array(ArrayObject { extensible, .. }) => *extensible,
        Object::Misc(misc) => misc
            .get_own_property(&PropertyKey::from("extensible"))
            .unwrap()
            .field("value")
            .unwrap()
            .unwrap_bool()
            .unwrap(),
    };

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
        Object::Misc(misc) => misc
            .get_own_property(&PropertyKey::from("extensible"))
            .unwrap()
            .field("value")
            .unwrap()
            .unwrap_bool()
            .unwrap(),
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
        Object::Misc(misc) => {
            let desc = misc
                .get_own_property(&PropertyKey::from("extensible"))
                .unwrap();

            if !desc.configurable() {
                return false;
            }

            misc.define_own_property(
                &PropertyKey::from("extensible"),
                PropertyDescriptor::Data {
                    value: Value::Boolean(false),
                    writable: false,
                    enumerable: false,
                    configurable: false,
                },
            );

            true
        }
    }
}

pub fn ordinary_get_own_property(object: &Object, key: &PropertyKey) -> Option<PropertyDescriptor> {
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

pub fn ordinary_define_own_property(
    object: &mut Object,
    key: &PropertyKey,
    desc: &PropertyDescriptor,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let _current = ordinary_get_own_property(object, key);
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

pub fn is_compatible_property_descriptor(
    extensible: bool,
    desc: &PropertyDescriptor,
    current: Option<&PropertyDescriptor>,
) -> bool {
    validate_and_apply_property_descriptor(None, &PropertyKey::empty(), extensible, desc, current)
}

pub fn validate_and_apply_property_descriptor(
    object: Option<&mut Object>,
    key: &PropertyKey,
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
                ordinary.properties.insert(key.clone(), desc.clone());
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
                        key.clone(),
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
                        key.clone(),
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

pub fn ordinary_set(
    object: &mut Object,
    key: &PropertyKey,
    value: &Value,
    receiver: &mut Value,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let own_desc = ordinary_get_own_property(object, key);

    if let Some(desc) = own_desc {
        ordinary_set_with_own_descriptor(object, key, value, receiver, Some(desc))
    } else {
        Err(CompletionRecordThrow(CompletionRecordError::Misc(format!(
            "Property {:?} does not exist on object",
            key
        ))))
    }
}

fn ordinary_set_with_own_descriptor(
    object: &mut Object,
    key: &PropertyKey,
    value: &Value,
    receiver: &mut Value,
    mut own_desc: Option<PropertyDescriptor>,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    if own_desc.is_none() {
        let prototype = object.get_prototype_of();
        let mut parent = prototype.borrow_mut();

        if let Some(parent_sure) = parent.as_mut() {
            return Ok(CompletionRecordNormal(
                parent_sure.set(key, value, receiver),
            ));
        }

        own_desc = Some(PropertyDescriptor::Data {
            value: Value::Undefined,
            writable: true,
            enumerable: true,
            configurable: true,
        });
    }

    if let Some(desc) = &own_desc
        && desc.is_data_descriptor()
    {
        if !desc
            .field("writable")
            .unwrap_or(Value::Boolean(true))
            .unwrap_bool()
            .unwrap()
        {
            return Ok(CompletionRecordNormal(false));
        }

        match receiver {
            Value::Object(_) => {}
            _ => return Ok(CompletionRecordNormal(false)),
        }

        let mut obj_receiver = receiver.unwrap_object().unwrap();
        let existing_desc = obj_receiver.get_own_property(key);

        if existing_desc.is_none() {
            let res = create_data_property(object, key, value)?;
            return Ok(res);
        }

        let existing_desc = existing_desc.unwrap();

        if existing_desc.is_accessor_descriptor() {
            return Ok(CompletionRecordNormal(false));
        }

        if !existing_desc
            .field("writable")
            .unwrap_or(Value::Boolean(true))
            .unwrap_bool()
            .unwrap()
        {
            return Ok(CompletionRecordNormal(false));
        }

        let value_desc_fields =
            HashMap::<String, Value>::from([(String::from("value"), value.clone())]);
        let value_desc = PropertyDescriptor::NonGeneric {
            fields: value_desc_fields,
        };

        let res = obj_receiver.define_own_property(key, value_desc);
        *receiver = Value::Object(obj_receiver);

        return Ok(CompletionRecordNormal(res));
    }

    let setter = own_desc.unwrap().field("set").unwrap_or(Value::Undefined);
    if matches!(setter, Value::Undefined) {
        return Ok(CompletionRecordNormal(false));
    }

    return todo!("Call");
}

pub fn ordinary_delete(
    object: &mut Object,
    key: &PropertyKey,
) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let desc = ordinary_get_own_property(object, key);

    if desc.is_none() {
        return Ok(CompletionRecordNormal(true));
    }

    let desc = desc.unwrap();
    if desc.configurable() {
        match object {
            Object::Ordinary(ordinary) => {
                ordinary.properties.remove(key);
            }
            _ => panic!(),
        }

        return Ok(CompletionRecordNormal(true));
    }

    Ok(CompletionRecordNormal(false))
}

//
// fn ordinary_own_property_keys(object: &Object) -> Vec<PropertyKey>;
