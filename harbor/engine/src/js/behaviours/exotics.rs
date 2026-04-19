// pub fn array_create(length: u32, )

pub mod array {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::js::{
        behaviours::{ordinary_define_own_property, ordinary_get_own_property},
        operations::{
            canonical_numeric_index_string, same_value_zero, to_number, to_object, to_uint32,
        },
        types::completion_record::{
            CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
            CompletionRecordThrow,
        },
        values::{
            Value,
            number::Number,
            object::{ArrayObject, Object, OrdinaryObject, PropertyDescriptor, PropertyKey},
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
                    internal_slots: HashMap::new(),
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
            let res = ordinary_define_own_property(&mut obj, &PropertyKey::from("length"), &desc);
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

        let old_len_desc = ordinary_get_own_property(&obj, &PropertyKey::from("length")).unwrap();
        let old_len = old_len_desc
            .field("value")
            .unwrap()
            .unwrap_number()
            .unwrap();

        if new_len >= old_len {
            let res =
                ordinary_define_own_property(&mut obj, &PropertyKey::from("length"), &new_len_desc);
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
            ordinary_define_own_property(&mut obj, &PropertyKey::from("length"), &new_len_desc)?;
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
                        &PropertyKey::from("length"),
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
                &PropertyKey::from("length"),
                &PropertyDescriptor::NonGeneric { fields },
            )?;
        }

        return Ok(CompletionRecordNormal(true));
    }

    pub enum IterationKind {
        KeyValue,
        Key,
        Value,
    }

    pub fn array_prototype_values_internal(args: Vec<Value>) -> Value {
        assert!(args.len() == 1);
        let this = &args[0];

        let _obj = to_object(this).unwrap().value;
        todo!("Implement array iterator prototype and shi")
    }
}

pub mod arguments {
    use std::{cell::RefCell, rc::Rc};

    use crate::js::{
        SLOT_EXTENSIBLE, SLOT_PARAMETER_MAP, SLOT_PROTOTYPE,
        behaviours::{
            _arguments_from_ordinary, _ordinary_from_misc,
            exotics::array::array_prototype_values_internal, ordinary_define_own_property,
            ordinary_delete, ordinary_get, ordinary_get_own_property, ordinary_get_prototype_of,
            ordinary_object_create, ordinary_set, ordinary_set_prototype_of,
        },
        executable::environment::EnvironmentRecord,
        operations::{
            create_data_property_or_throw, define_property_or_throw, get, has_own_property,
            make_basic_object, same_value, set,
        },
        semantics::r#static::{ParseNode, StaticSemantics},
        values::{
            Value,
            number::Number,
            object::{Object, ObjectTrait, OrdinaryObject, PropertyDescriptor, PropertyKey},
            symbol::SYMBOL_ITERATOR,
        },
    };

    #[derive(Clone, Debug)]
    pub struct ArgumentsObject {
        pub ordinary: OrdinaryObject,
        pub parameter_map: OrdinaryObject,
    }

    impl ObjectTrait for ArgumentsObject {
        const CALLABLE: bool = false;
        const CONSTRUCTOR: bool = false;

        fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
            ordinary_get_prototype_of(&Object::Arguments(self.clone()))
        }

        fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
            let mut obj = Object::Arguments(self.clone());
            let res = ordinary_set_prototype_of(&mut obj, prototype);

            if let Object::Arguments(args) = obj {
                *self = args;
            }

            res
        }

        fn has_property(&self, key: &PropertyKey) -> bool {
            let desc = self.get_own_property(key);
            if desc.is_some() {
                return true;
            }

            let proto = self.get_prototype_of();
            if let Some(proto) = proto.borrow().as_ref() {
                return proto.has_property(key);
            }

            false
        }

        fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
            let desc = ordinary_get_own_property(&Object::Arguments(self.clone()), key);
            if desc.is_none() {
                return None;
            }

            let mut desc = desc.unwrap();

            let map = &self.parameter_map;
            let map_object = Object::Ordinary(map.clone());

            let is_mapped = has_own_property(&map_object, key).unwrap().value;

            if is_mapped {
                desc.set_field("value", get(&map_object, key).unwrap().value);
            }

            return Some(desc);
        }

        fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
            let mut obj = Object::Arguments(self.clone());

            let map = &self.parameter_map;
            let mut map_object = Object::Ordinary(map.clone());

            let is_mapped = has_own_property(&map_object, key).unwrap().value;
            let new_args_desc = &desc;

            let allowed = ordinary_define_own_property(&mut obj, key, new_args_desc)
                .unwrap()
                .value;

            // NOTE: TV Girl reference?
            if !allowed {
                return false;
            }

            if is_mapped {
                if desc.is_accessor_descriptor() {
                    map_object.delete(key);
                } else {
                    if desc.field("value").is_some() {
                        set(&mut map_object, key, &desc.field("value").unwrap(), false).unwrap();
                    }

                    if desc
                        .field("writable")
                        .is_some_and(|v| v.unwrap_bool().unwrap() == false)
                    {
                        map_object.delete(key);
                    }
                }
            }

            if let Object::Arguments(args) = obj {
                *self = args;
            }

            if let Object::Ordinary(map) = map_object {
                self.parameter_map = map;
            }

            true
        }

        fn get(&self, key: &PropertyKey, receiver: &Value) -> Option<Value> {
            let map = &self.parameter_map;
            let map_object = Object::Ordinary(map.clone());

            let is_mapped = has_own_property(&map_object, key).unwrap().value;
            if is_mapped {
                return ordinary_get(&map_object, key, receiver)
                    .ok()
                    .map(|v| v.value);
            }

            get(&map_object, key).ok().map(|v| v.value)
        }

        fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
            let mut args_object = Object::Arguments(self.clone());

            let map = &self.parameter_map;
            let mut map_object = Object::Ordinary(map.clone());

            let is_mapped = if !same_value(&Value::Object(args_object.clone()), receiver) {
                false
            } else {
                has_own_property(&map_object, key).unwrap().value
            };

            if is_mapped {
                set(&mut map_object, key, value, false).unwrap();
                if let Object::Ordinary(map) = &map_object {
                    self.parameter_map = map.clone();
                    args_object = Object::Arguments(self.clone());
                }
            }

            let res = ordinary_set(&mut args_object, key, value, receiver);

            if let Object::Arguments(args) = args_object {
                *self = args;
            }

            res.unwrap().value
        }

        fn delete(&mut self, key: &PropertyKey) -> bool {
            let mut args_object = Object::Arguments(self.clone());

            let map = &self.parameter_map;
            let mut map_object = Object::Ordinary(map.clone());

            let is_mapped = has_own_property(&map_object, key).unwrap().value;

            if is_mapped {
                map_object.delete(key);
                if let Object::Ordinary(map) = &map_object {
                    self.parameter_map = map.clone();
                    args_object = Object::Arguments(self.clone());
                }
            }

            let res = ordinary_delete(&mut args_object, key);

            if let Object::Arguments(args) = args_object {
                *self = args;
            }

            res.unwrap().value
        }

        fn call(&self, _this: &Value, _args: Vec<Value>) -> Value {
            panic!("Arguments object is not callable");
        }

        fn construct(&self, _args: Vec<Value>, _new_target: &Object) -> Object {
            panic!("Arguments object is not a constructor");
        }
    }

    pub fn create_unmapped_arguments_object(args: &Vec<Value>) -> OrdinaryObject {
        let len = args.len();
        let mut obj = ordinary_object_create(
            Some(Object::Ordinary(OrdinaryObject::prototype())),
            vec![String::from(SLOT_PARAMETER_MAP)],
        );

        obj.properties.insert(
            PropertyKey::from(SLOT_PARAMETER_MAP),
            PropertyDescriptor::data_descriptor(Value::Undefined, false, false, false),
        );

        let mut obj = Object::Ordinary(obj);

        define_property_or_throw(
            &mut obj,
            &PropertyKey::from("length"),
            PropertyDescriptor::data_descriptor(
                Value::Number(Number(len as f64)),
                true,
                false,
                true,
            ),
        )
        .unwrap();

        let mut index = 0;
        while index < len {
            let val = args.get(index).unwrap_or(&Value::Undefined);
            create_data_property_or_throw(&mut obj, &PropertyKey::from(index.to_string()), val)
                .unwrap();

            index += 1;
        }

        define_property_or_throw(
            &mut obj,
            &PropertyKey::Symbol(SYMBOL_ITERATOR.clone()),
            PropertyDescriptor::data_descriptor(
                Value::InternalFunction(Rc::new(array_prototype_values_internal)),
                true,
                false,
                true,
            ),
        )
        .unwrap();

        // TODO: implement 'callee' prop

        if let Object::Ordinary(obj) = obj {
            obj
        } else {
            unreachable!("Expected ordinary object");
        }
    }

    pub fn create_mapped_arguments_object(
        func: &Object,
        formals: ParseNode,
        args_list: &Vec<Value>,
        _env: Rc<RefCell<EnvironmentRecord>>,
    ) -> ArgumentsObject {
        let len = args_list.len();

        if let Object::Misc(misc) = make_basic_object(vec![
            String::from(SLOT_PROTOTYPE),
            String::from(SLOT_EXTENSIBLE),
            String::from(SLOT_PARAMETER_MAP),
        ]) {
            let obj_raw = _ordinary_from_misc(&misc);
            let mut args = _arguments_from_ordinary(&obj_raw);

            args.parameter_map = ordinary_object_create(
                Some(Object::Ordinary(OrdinaryObject::prototype())),
                Vec::new(),
            );

            let param_names = formals.bound_names();
            let number_of_params = param_names.len();

            let mut index = 0;

            let mut obj = Object::Arguments(args.clone());

            while index < len {
                let val = args_list.get(index).unwrap();
                create_data_property_or_throw(
                    &mut obj,
                    &PropertyKey::from(index.to_string()),
                    &val,
                )
                .unwrap();

                index += 1;
            }

            define_property_or_throw(
                &mut obj,
                &PropertyKey::from("length"),
                PropertyDescriptor::data_descriptor(
                    Value::Number(Number(len as f64)),
                    true,
                    false,
                    true,
                ),
            )
            .unwrap();

            let mut mapped_names = Vec::new();

            if number_of_params > 0 {
                index = number_of_params - 1;

                loop {
                    let name = &param_names[index];
                    if !mapped_names.contains(name) {
                        mapped_names.push(name.clone());

                        if index < len {
                            // TODO: implement environment record bindings and stuff
                        }
                    }

                    if index == 0 {
                        break;
                    }
                    index -= 1;
                }
            }

            define_property_or_throw(
                &mut obj,
                &PropertyKey::Symbol(SYMBOL_ITERATOR.clone()),
                PropertyDescriptor::data_descriptor(
                    Value::InternalFunction(Rc::new(array_prototype_values_internal)),
                    true,
                    false,
                    true,
                ),
            )
            .unwrap();

            define_property_or_throw(
                &mut obj,
                &PropertyKey::from("callee"),
                PropertyDescriptor::data_descriptor(Value::Object(func.clone()), true, false, true),
            )
            .unwrap();

            if let Object::Arguments(args) = obj {
                args
            } else {
                unreachable!("Expected arguments object");
            }
        } else {
            unreachable!("Expected misc object");
        }
    }
}
