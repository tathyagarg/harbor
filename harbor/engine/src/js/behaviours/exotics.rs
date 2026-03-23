// pub fn array_create(length: u32, )

pub mod array {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::js::{
        behaviours::{ordinary_define_own_property, ordinary_get_own_property},
        operations::{canonical_numeric_index_string, to_number, to_uint32},
        types::completion_record::{
            CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
            CompletionRecordThrow,
        },
        values::{
            Value,
            number::Number,
            object::{ArrayObject, Object, OrdinaryObject, PropertyDescriptor, PropertyKey},
            same_value_zero,
        },
    };

    impl ArrayObject {
        pub fn prototype() -> Object {
            let mut properties = HashMap::<PropertyKey, PropertyDescriptor>::new();
            properties.insert(
                PropertyKey::from("length"),
                PropertyDescriptor::data_descriptor(Value::Number(Number(0.0)), true, false, false),
            );

            Object::Array(ArrayObject {
                length: 0,
                extensible: true,
                data: Vec::new(),
                object: OrdinaryObject {
                    properties,
                    prototype: Rc::new(RefCell::new(Some(Object::prototype()))),
                    extensible: true,
                },
            })
        }

        pub fn define_own_property(
            &mut self,
            key: PropertyKey,
            desc: PropertyDescriptor,
        ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>>
        {
            if key == "length" {
                return array_set_length(self, desc);
            }

            if let PropertyKey::String(s) = &key
                && canonical_numeric_index_string(s).is_some()
            {}

            todo!()
        }

        pub fn delete(
            &mut self,
            key: &PropertyKey,
        ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>>
        {
            // ensure key is an int
            let int_value = to_uint32(Value::String(key.into()));

            if let Ok(int) = int_value {
                let index = int.unwrapped();

                if (index.0 as u32) < self.object.properties.len() as u32 {
                    let prop_key = PropertyKey::from(index.to_string(10));
                    if self.object.properties.contains_key(&prop_key) {
                        self.object.properties.remove(&prop_key);
                        return Ok(CompletionRecordNormal(true));
                    }
                }
            } else {
                // if key is not an int, delete it normally
                // return self.object.delete(key);
                todo!("Implement delete for ordinary");
                // return Ok(CompletionRecordNormal(true));
            }

            return Ok(CompletionRecordNormal(false));
        }
    }

    pub fn array_set_length(
        array: &mut ArrayObject,
        desc: PropertyDescriptor,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError, CRKThrow>> {
        let mut obj = Object::Array(array.clone());

        if desc.field("value").is_none() {
            let res = ordinary_define_own_property(&mut obj, PropertyKey::from("length"), &desc);
            if let Object::Array(arr) = obj {
                *array = arr;
            }

            return res;
        }

        let mut new_len_desc = desc.clone();

        let new_len_rec = to_uint32(desc.field("value").unwrap())?;
        let new_len = new_len_rec.unwrapped().clone();

        let number_len_rec = to_number(desc.field("value").unwrap())?;
        let number_len = number_len_rec.unwrapped();

        if !same_value_zero(&Value::Number(new_len), &Value::Number(*number_len)) {
            return Err(CompletionRecordThrow(CompletionRecordError::RangeErorr));
        }

        new_len_desc.set_field("value", Value::Number(new_len));

        let old_len_desc = ordinary_get_own_property(&obj, PropertyKey::from("length")).unwrap();
        let old_len = old_len_desc
            .field("value")
            .unwrap()
            .unwrap_number()
            .unwrap();

        if new_len >= old_len {
            let res =
                ordinary_define_own_property(&mut obj, PropertyKey::from("length"), &new_len_desc);
            if let Object::Array(arr) = obj {
                *array = arr;
            }

            return res;
        }

        if !old_len_desc
            .field("writable")
            .unwrap_or(Value::Boolean(true))
            .unwrap_bool()
            .unwrap()
        {
            return Ok(CompletionRecordNormal(false));
        }

        let new_writable = if new_len_desc.field("writable").is_none()
            || new_len_desc
                .field("writable")
                .unwrap()
                .unwrap_bool()
                .unwrap()
        {
            true
        } else {
            new_len_desc.set_field("writable", Value::Boolean(true));
            false
        };

        let succeeded =
            ordinary_define_own_property(&mut obj, PropertyKey::from("length"), &new_len_desc)?;
        if let Object::Array(arr) = &obj {
            *array = arr.clone();
        }

        if !succeeded.unwrapped() {
            return Ok(CompletionRecordNormal(false));
        }

        for (prop_key, _) in array.object.properties.clone() {
            if to_uint32(Value::String(prop_key.clone().into()))
                .unwrap()
                .unwrapped()
                >= &new_len
            {
                let delete_succeeded = *(array.delete(&prop_key)?.unwrapped());
                if !delete_succeeded {
                    new_len_desc.set_field(
                        "value",
                        Value::Number(
                            *to_number(Value::String(prop_key.clone().into()))?.unwrapped()
                                + 1.0f64,
                        ),
                    );

                    if !new_writable {
                        new_len_desc.set_field("writable", Value::Boolean(false));
                    }

                    ordinary_define_own_property(
                        &mut obj,
                        PropertyKey::from("length"),
                        &new_len_desc,
                    )?;
                    if let Object::Array(arr) = &obj {
                        *array = arr.clone();
                    }

                    return Ok(CompletionRecordNormal(false));
                }
            }
        }

        if !new_writable {
            let mut fields = HashMap::<String, Value>::new();
            fields.insert(String::from("writable"), Value::Boolean(false));

            ordinary_define_own_property(
                &mut obj,
                PropertyKey::from("length"),
                &PropertyDescriptor::NonGeneric { fields },
            )?;
        }

        return Ok(CompletionRecordNormal(true));
    }
}
