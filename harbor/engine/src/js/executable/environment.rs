use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::js::{
    types::completion_record::{
        CRKThrow, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
        CompletionRecordThrow, UNUSED,
    },
    values::{
        Value,
        object::{FunctionObject, Object, ThisMode},
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

impl EnvironmentRecord {
    pub fn has_binding(
        &self,
        name: &JsString,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<UNUSED, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => {
                let has_binding = self.bindings.contains_key(&name);

                Ok(CompletionRecordNormal(has_binding))
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record.borrow().has_binding(name),
            _ => todo!(
                "has_binding is only implemented for declarative environment records, not {:?} (for binding: {:?})",
                self.kind,
                name
            ),
        }
    }

    pub fn create_mutable_binding(
        &mut self,
        name: JsString,
        deletable: bool,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => {
                if self.bindings.contains_key(&name) && !self.bindings[&name].deletable {
                    return Err(CompletionRecordThrow(()));
                }

                self.bindings.insert(
                    name,
                    Binding {
                        value: Value::empty(),
                        deletable,
                        mutable: true,
                        strict: false,
                        initialized: false,
                    },
                );

                Ok(CompletionRecordNormal(()))
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record
                .borrow_mut()
                .create_mutable_binding(name, deletable),
            _ => todo!(),
        }
    }

    pub fn create_immutable_binding(
        &mut self,
        name: JsString,
        strict: bool,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => {
                if self.bindings.contains_key(&name) {
                    return Err(CompletionRecordThrow(()));
                }

                self.bindings.insert(
                    name,
                    Binding {
                        value: Value::empty(),
                        deletable: false,
                        mutable: false,
                        strict,
                        initialized: false,
                    },
                );

                Ok(CompletionRecordNormal(()))
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record
                .borrow_mut()
                .create_immutable_binding(name, strict),
            _ => todo!(),
        }
    }

    pub fn initialize_binding(
        &mut self,
        name: JsString,
        value: &Value,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => {
                if !self.bindings.contains_key(&name) {
                    return Err(CompletionRecordThrow(()));
                }

                let binding = self.bindings.get_mut(&name).unwrap();
                if binding.initialized {
                    return Err(CompletionRecordThrow(()));
                }

                binding.value = value.clone();
                binding.initialized = true;

                Ok(CompletionRecordNormal(()))
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record
                .borrow_mut()
                .initialize_binding(name, value),
            _ => todo!(),
        }
    }

    pub fn set_mutable_binding(
        &mut self,
        name: JsString,
        value: Value,
        strict: bool,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => {
                if !self.bindings.contains_key(&name) {
                    return Err(CompletionRecordThrow(()));
                }

                let binding = self.bindings.get_mut(&name).unwrap();
                if !binding.initialized {
                    return Err(CompletionRecordThrow(()));
                }

                binding.value = value;

                Ok(CompletionRecordNormal(()))
            }
            EnvironmentRecordKind::Global {
                declarative_record, ..
            } => declarative_record
                .borrow_mut()
                .set_mutable_binding(name, value, strict),
            _ => todo!(),
        }
    }

    pub fn get_binding_value(
        &self,
        name: JsString,
        strict: bool,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKThrow>> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => {
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
                declarative_record, ..
            } => declarative_record.borrow().get_binding_value(name, strict),
            _ => todo!(),
        }
    }

    pub fn delete_binding(
        &mut self,
        name: JsString,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<UNUSED, CRKThrow>> {
        match self.kind {
            EnvironmentRecordKind::Declarative => {
                if !self.bindings.contains_key(&name) {
                    return Ok(CompletionRecordNormal(true));
                }

                let binding = self.bindings.get(&name).unwrap();
                if !binding.deletable {
                    return Ok(CompletionRecordNormal(false));
                }

                self.bindings.remove(&name);
                Ok(CompletionRecordNormal(true))
            }
            _ => todo!(),
        }
    }

    pub fn with_base_object(&self) -> Option<Object> {
        match &self.kind {
            EnvironmentRecordKind::Declarative => None,
            EnvironmentRecordKind::Object(obj_rec) => {
                if obj_rec.is_with_environment {
                    Some(obj_rec.object.borrow().clone())
                } else {
                    None
                }
            }
            EnvironmentRecordKind::Function { .. } => None,
            EnvironmentRecordKind::Module => None,
            EnvironmentRecordKind::Global { .. } => None,
        }
    }

    // TODO: Implement other methods (this, super based methods)
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
            let exists = env.borrow().has_binding(&name.clone())?;
            if exists.value {
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
        outer_env: Some(func.environment.clone()),
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
