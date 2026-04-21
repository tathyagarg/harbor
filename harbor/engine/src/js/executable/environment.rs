use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::js::{
    operations::{define_property_or_throw, get, has_own_property, has_property, set},
    types::completion_record::{
        CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
        CompletionRecordThrow,
    },
    values::{
        Value,
        object::{FunctionObject, Object, ObjectTrait, PropertyDescriptor, PropertyKey, ThisMode},
        reference::{Reference, ReferenceBase, ReferenceName},
        string::JsString,
    },
};

#[derive(Clone, Debug)]
pub enum BindingStatus {
    Lexical,
    Initialized,
    Uninitialized,
}

#[derive(Clone, Debug)]
pub struct ObjectEnvironmentRecord {
    pub object: Rc<RefCell<Object>>,

    pub is_with_environment: bool,
}

#[derive(Clone, Debug)]
pub enum EnvironmentRecordKind {
    Declarative,
    Object(ObjectEnvironmentRecord),
    Function {
        this_value: Rc<Value>,
        this_binding_status: BindingStatus,
        function_object: Rc<FunctionObject>,
        new_target: Option<Rc<FunctionObject>>,
    },
    Module,
    Global {
        object: ObjectEnvironmentRecord,
        global_this_value: Rc<RefCell<Object>>,
        declarative_record: Rc<RefCell<EnvironmentRecord>>,
    },
}

// WARN: This note may not be accurate
// NOTE: This is a placeholder type. In a complete implementation, this would likely be a more
// complex type that can represent any JavaScript value, such as a union of different types or an
// enum.
#[derive(Clone, Debug)]
pub struct Binding {
    pub value: Value,

    pub deletable: bool,
    pub mutable: bool,
    pub strict: bool,
    pub initialized: bool,
}

#[derive(Clone, Debug)]
pub struct EnvironmentRecord {
    /// NOTE: This is not a Option<Box<EnvironmentRecord>> because, according to the spec:
    /// > An Environment Record may serve as the outer environment for multiple inner Environment Records
    /// [^]: https://tc39.es/ecma262/#sec-environment-records
    /// Such a structure is not possible with Box, but Rc allows for multiple ownership.
    pub outer_env: Option<Rc<RefCell<EnvironmentRecord>>>,

    pub kind: EnvironmentRecordKind,

    pub bindings: HashMap<JsString, Binding>,
}

pub trait EnvRecordTrait {
    fn has_binding(&self, name: &JsString) -> bool;
    fn create_mutable_binding(&mut self, name: &JsString, deletable: bool);
    fn create_immutable_binding(&mut self, name: &JsString, strict: bool);
    fn initialize_binding(&mut self, name: &JsString, value: &Value);
    fn set_mutable_binding(&mut self, name: &JsString, value: &Value, strict: bool);
    fn get_binding_value(
        &self,
        name: &JsString,
        strict: bool,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>>;
    fn delete_binding(&mut self, name: &JsString) -> bool;
    fn has_this_binding(&self) -> bool;
    fn get_this_binding(&self) -> Value;
    fn has_super_binding(&self) -> bool;
    fn with_base_object(&self) -> Option<Object>;
}

impl EnvRecordTrait for ObjectEnvironmentRecord {
    fn has_binding(&self, name: &JsString) -> bool {
        let binding_obj = self.object.borrow();
        let found_binding = has_property(&binding_obj, &PropertyKey::String(name.clone()))
            .unwrap()
            .value;

        if !found_binding {
            return false;
        }

        if !self.is_with_environment {
            return true;
        }

        // TODO: unscopables

        true
    }

    fn create_mutable_binding(&mut self, name: &JsString, deletable: bool) {
        let mut binding_obj = self.object.borrow_mut();
        define_property_or_throw(
            &mut binding_obj,
            &PropertyKey::String(name.clone()),
            PropertyDescriptor::Data {
                value: Value::Undefined,
                writable: true,
                enumerable: true,
                configurable: deletable,
            },
        )
        .unwrap();
    }

    /// WARN: This function is never used:
    /// https://tc39.es/ecma262/#sec-object-environment-records-createimmutablebinding-n-s
    fn create_immutable_binding(&mut self, _name: &JsString, _strict: bool) {
        panic!("This is never used");
    }

    fn initialize_binding(&mut self, name: &JsString, value: &Value) {
        self.set_mutable_binding(name, value, false);
    }

    fn set_mutable_binding(&mut self, name: &JsString, value: &Value, strict: bool) {
        let still_exists = has_property(&self.object.borrow(), &PropertyKey::String(name.clone()))
            .unwrap()
            .value;

        if !still_exists && strict {
            panic!("Cannot set value of non-existent binding in strict mode");
        }

        set(
            &mut self.object.borrow_mut(),
            &PropertyKey::String(name.clone()),
            value,
            strict,
        )
        .unwrap();
    }

    fn get_binding_value(
        &self,
        name: &JsString,
        strict: bool,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
        let binding_obj = self.object.borrow();
        let value = has_property(&binding_obj, &PropertyKey::String(name.clone()))
            .unwrap()
            .value;

        if !value {
            if strict {
                return Err(CompletionRecordThrow(CompletionRecordError::ReferenceError));
            } else {
                return Ok(CompletionRecordNormal(Value::Undefined));
            }
        }

        get(&binding_obj, &PropertyKey::String(name.clone()))
    }

    fn delete_binding(&mut self, name: &JsString) -> bool {
        let mut binding_obj = self.object.borrow_mut();
        binding_obj.delete(&PropertyKey::String(name.clone()))
    }

    fn has_this_binding(&self) -> bool {
        false
    }

    /// WARN: This function is never used:
    /// https://tc39.es/ecma262/#sec-object-environment-records-getthisbinding
    fn get_this_binding(&self) -> Value {
        panic!("Object environment records do not have a this binding");
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Option<Object> {
        if self.is_with_environment {
            Some(self.object.borrow().clone())
        } else {
            None
        }
    }
}

impl EnvRecordTrait for EnvironmentRecord {
    fn has_binding(&self, name: &JsString) -> bool {
        match &self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                let has_binding = self.bindings.contains_key(&name);

                has_binding
            }
            EnvironmentRecordKind::Global {
                declarative_record,
                object,
                ..
            } => {
                let res = declarative_record.borrow().has_binding(name);
                if res {
                    return true;
                }

                object.has_binding(name)
            }
            EnvironmentRecordKind::Object(obj_rec) => obj_rec.has_binding(name),
            _ => todo!(
                "has_binding is only implemented for declarative environment records, not {:?} (for binding: {:?})",
                self.kind,
                name
            ),
        }
    }

    fn create_mutable_binding(&mut self, name: &JsString, deletable: bool) {
        match &mut self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                if self.bindings.contains_key(&name) && !self.bindings[&name].deletable {
                    return;
                }

                self.bindings.insert(
                    name.clone(),
                    Binding {
                        value: Value::empty(),
                        deletable,
                        mutable: true,
                        strict: false,
                        initialized: false,
                    },
                );

                return;
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record
                .borrow_mut()
                .create_mutable_binding(name, deletable),
            EnvironmentRecordKind::Object(obj_rec) => {
                obj_rec.create_mutable_binding(name, deletable)
            }
            _ => todo!(
                "create_mutable_binding is only implemented for declarative environment records, not {:?} (for binding: {:?})",
                self.kind,
                name
            ),
        }
    }

    fn create_immutable_binding(&mut self, name: &JsString, strict: bool) {
        match &mut self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                if self.bindings.contains_key(&name) {
                    return;
                }

                self.bindings.insert(
                    name.clone(),
                    Binding {
                        value: Value::empty(),
                        deletable: false,
                        mutable: false,
                        strict,
                        initialized: false,
                    },
                );
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record
                .borrow_mut()
                .create_immutable_binding(name, strict),
            EnvironmentRecordKind::Object(obj_rec) => {
                obj_rec.create_immutable_binding(name, strict)
            }
            _ => todo!(),
        }
    }

    fn initialize_binding(&mut self, name: &JsString, value: &Value) {
        match &mut self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                if !self.bindings.contains_key(&name) {
                    return;
                }

                let binding = self.bindings.get_mut(&name).unwrap();
                if binding.initialized {
                    return;
                }

                binding.value = value.clone();
                binding.initialized = true;

                return;
            }
            EnvironmentRecordKind::Global {
                declarative_record,
                object,
                ..
            } => {
                let has_binding = declarative_record.borrow().has_binding(name);
                if has_binding {
                    return declarative_record
                        .borrow_mut()
                        .initialize_binding(name, value);
                }

                object.initialize_binding(name, value)
            }
            _ => todo!(),
        }
    }

    fn set_mutable_binding(&mut self, name: &JsString, value: &Value, strict: bool) {
        match &mut self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                if !self.bindings.contains_key(&name) {
                    return;
                }

                let binding = self.bindings.get_mut(&name).unwrap();
                if !binding.initialized {
                    return;
                }

                binding.value = value.clone();

                return;
            }
            EnvironmentRecordKind::Global {
                declarative_record,
                object,
                ..
            } => {
                let has_binding = declarative_record.borrow().has_binding(name);
                if has_binding {
                    return declarative_record
                        .borrow_mut()
                        .set_mutable_binding(name, value, strict);
                }

                object.set_mutable_binding(name, value, strict);
            }
            EnvironmentRecordKind::Object(obj_rec) => {
                obj_rec.set_mutable_binding(name, value, strict);
            }
            _ => todo!(),
        }
    }

    fn get_binding_value(
        &self,
        name: &JsString,
        strict: bool,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                if !self.bindings.contains_key(&name) {
                    if strict {
                        return Err(CompletionRecordThrow(CompletionRecordError::ReferenceError));
                    } else {
                        return Ok(CompletionRecordNormal(Value::Undefined));
                    }
                }

                let binding = self.bindings.get(&name).unwrap();
                if !binding.initialized {
                    return Err(CompletionRecordThrow(CompletionRecordError::ReferenceError));
                }

                Ok(CompletionRecordNormal(binding.value.clone()))
            }
            EnvironmentRecordKind::Global {
                declarative_record,
                object,
                ..
            } => {
                if declarative_record.borrow().has_binding(&name) {
                    return declarative_record.borrow().get_binding_value(name, strict);
                }

                object.get_binding_value(name, strict)
            }
            EnvironmentRecordKind::Object(obj_rec) => obj_rec.get_binding_value(name, strict),
            _ => todo!(),
        }
    }

    fn delete_binding(&mut self, name: &JsString) -> bool {
        match &mut self.kind {
            EnvironmentRecordKind::Declarative | EnvironmentRecordKind::Function { .. } => {
                if !self.bindings.contains_key(&name) {
                    return true;
                }

                let binding = self.bindings.get(&name).unwrap();
                if !binding.deletable {
                    return false;
                }

                self.bindings.remove(&name);
                true
            }
            EnvironmentRecordKind::Global {
                declarative_record,
                object,
                ..
            } => {
                if declarative_record.borrow().has_binding(&name) {
                    return declarative_record.borrow_mut().delete_binding(name);
                }

                let global_obj = &object.object;
                let existing_prop =
                    has_own_property(&global_obj.borrow(), &PropertyKey::String(name.clone()))
                        .unwrap()
                        .value;

                if existing_prop {
                    return object.delete_binding(name);
                }

                true
            }
            EnvironmentRecordKind::Object(obj_rec) => obj_rec.delete_binding(name),
            _ => todo!(),
        }
    }

    fn with_base_object(&self) -> Option<Object> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => None,
            EnvironmentRecordKind::Object(obj_rec) => obj_rec.with_base_object(),
            EnvironmentRecordKind::Function { .. } => None,
            EnvironmentRecordKind::Module => None,
            EnvironmentRecordKind::Global { .. } => None,
        }
    }

    fn has_this_binding(&self) -> bool {
        todo!()
    }

    fn get_this_binding(&self) -> Value {
        todo!()
    }

    fn has_super_binding(&self) -> bool {
        todo!()
    }
}

pub fn get_identifier_reference(
    name: JsString,
    env: Option<Rc<RefCell<EnvironmentRecord>>>,
    strict: bool,
) -> Result<CompletionRecord<Reference>, CompletionRecord<(), CRKThrow>> {
    match env {
        None => Ok(CompletionRecordNormal(Reference {
            base: ReferenceBase::Unresolvable,
            referenced_name: ReferenceName::Value(Value::String(name)),
            strict,
            this_value: None,
        })),
        Some(env) => {
            let exists = env.borrow().has_binding(&name.clone());
            if exists {
                Ok(CompletionRecordNormal(Reference {
                    base: ReferenceBase::EnvironmentRecord(env),
                    referenced_name: ReferenceName::Value(Value::String(name)),
                    strict,
                    this_value: None,
                }))
            } else {
                get_identifier_reference(name, env.borrow().outer_env.clone(), strict)
            }
        }
    }
}

pub fn new_declarative_environment(
    outer_env: Option<Rc<RefCell<EnvironmentRecord>>>,
) -> EnvironmentRecord {
    EnvironmentRecord {
        bindings: HashMap::new(),
        outer_env,
        kind: EnvironmentRecordKind::Declarative,
    }
}

pub fn new_object_environment(
    obj: &Rc<RefCell<Object>>,
    is_with_environment: bool,
    outer_env: Option<Rc<RefCell<EnvironmentRecord>>>,
) -> EnvironmentRecord {
    EnvironmentRecord {
        bindings: HashMap::new(),
        outer_env,
        kind: EnvironmentRecordKind::Object(ObjectEnvironmentRecord {
            object: obj.clone(),
            is_with_environment,
        }),
    }
}

pub fn new_function_environment(
    func: &FunctionObject,
    new_target: Option<Object>,
) -> EnvironmentRecord {
    EnvironmentRecord {
        outer_env: Some(func.environment.upgrade().unwrap()),
        kind: EnvironmentRecordKind::Function {
            function_object: Rc::new(func.clone()),
            this_binding_status: if matches!(func.this_mode, ThisMode::Lexical) {
                BindingStatus::Lexical
            } else {
                BindingStatus::Uninitialized
            },
            new_target: new_target.map(|obj| {
                if let Object::Function(func) = obj {
                    Rc::new(func)
                } else {
                    panic!("new_target must be a function object");
                }
            }),
            this_value: Rc::new(Value::Undefined),
        },
        bindings: HashMap::new(),
    }
}

pub fn new_global_environment(
    global_object: &Rc<RefCell<Object>>,
    this_value: &Rc<RefCell<Object>>,
) -> EnvironmentRecord {
    let obj_rec_wrapped = new_object_environment(global_object, false, None);
    let obj_rec = match obj_rec_wrapped.kind {
        EnvironmentRecordKind::Object(obj_rec) => obj_rec,
        _ => unreachable!(),
    };

    let dcl_rec = new_declarative_environment(None);

    EnvironmentRecord {
        outer_env: None,
        bindings: HashMap::new(),
        kind: EnvironmentRecordKind::Global {
            object: obj_rec,
            global_this_value: this_value.clone(),
            declarative_record: Rc::new(RefCell::new(dcl_rec)),
        },
    }
}

pub fn bind_this_value(
    env: Rc<RefCell<EnvironmentRecord>>,
    value: &Value,
) -> Result<CompletionRecord<()>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let mut env_borrow = env.borrow_mut();
    match &mut env_borrow.kind {
        EnvironmentRecordKind::Function {
            this_binding_status,
            this_value,
            ..
        } => {
            if matches!(this_binding_status, BindingStatus::Lexical) {
                return Err(CompletionRecordThrow(CompletionRecordError::ReferenceError));
            }

            *this_value = Rc::new(value.clone());
            *this_binding_status = BindingStatus::Initialized;

            Ok(CompletionRecordNormal(()))
        }
        _ => panic!("bind_this_value can only be called on function environment records"),
    }
}

pub fn create_global_function_binding(
    env_rec: Rc<RefCell<EnvironmentRecord>>,
    name: JsString,
    value: &Value,
    configurable: bool,
) -> Result<CompletionRecord<()>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let mut env_rec_borrow = env_rec.borrow_mut();
    match &mut env_rec_borrow.kind {
        EnvironmentRecordKind::Global {
            object: obj_rec, ..
        } => {
            let global_object = obj_rec.object.clone();
            let existing_prop = global_object
                .borrow()
                .get_own_property(&PropertyKey::String(name.clone()));

            let desc = if existing_prop.is_none() || existing_prop.unwrap().configurable() {
                PropertyDescriptor::Data {
                    value: value.clone(),
                    writable: true,
                    enumerable: true,
                    configurable,
                }
            } else {
                let mut fields = HashMap::new();
                fields.insert(String::from("value"), value.clone());

                PropertyDescriptor::NonGeneric { fields }
            };

            define_property_or_throw(
                &mut global_object.borrow_mut(),
                &PropertyKey::String(name.clone()),
                desc,
            )?;

            set(
                &mut global_object.borrow_mut(),
                &PropertyKey::String(name),
                value,
                false,
            )?;

            Ok(CompletionRecordNormal(()))
        }
        _ => panic!(
            "create_global_function_binding can only be called on global environment records"
        ),
    }
}

pub fn create_global_var_binding(
    env_rec: Rc<RefCell<EnvironmentRecord>>,
    name: JsString,
    deletable: bool,
) -> Result<CompletionRecord<()>, CompletionRecord<CompletionRecordError, CRKThrow>> {
    let mut env_rec_borrow = env_rec.borrow_mut();
    match &mut env_rec_borrow.kind {
        EnvironmentRecordKind::Global {
            object: obj_rec, ..
        } => {
            let global_object = obj_rec.object.clone();
            let has_prop =
                has_own_property(&global_object.borrow(), &PropertyKey::String(name.clone()))
                    .unwrap()
                    .value;

            if !has_prop {
                define_property_or_throw(
                    &mut global_object.borrow_mut(),
                    &PropertyKey::String(name.clone()),
                    PropertyDescriptor::Data {
                        value: Value::Undefined,
                        writable: true,
                        enumerable: true,
                        configurable: deletable,
                    },
                )?;

                set(
                    &mut global_object.borrow_mut(),
                    &PropertyKey::String(name.clone()),
                    &Value::Undefined,
                    false,
                )?;
            }

            Ok(CompletionRecordNormal(()))
        }
        _ => panic!("create_global_var_binding can only be called on global environment records"),
    }
}
