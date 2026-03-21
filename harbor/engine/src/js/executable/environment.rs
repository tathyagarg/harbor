use std::{collections::HashMap, hash::Hash, rc::Rc};

use crate::js::{
    types::completion_record::{
        CompletionRecord, CompletionRecordError, CompletionRecordNormal, CompletionRecordThrow,
        UNUSED,
    },
    values::{Reference, ReferenceBase, Value, string::JsString},
};

#[derive(Clone, Debug)]
pub enum EnvironmentRecordKind {
    Declarative,
    Object {
        // NOTE: In a complete implementation, this would likely be a more complex type that can
        // represent any JavaScript object
        object: String,

        is_with_environment: bool,
    },
    Function,
    Module,
    Global,
}

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
    pub outer_env: Option<Rc<EnvironmentRecord>>,

    pub kind: EnvironmentRecordKind,

    pub bindings: HashMap<JsString, Binding>,
}

impl EnvironmentRecord {
    pub fn has_binding(
        &self,
        name: JsString,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<UNUSED>> {
        match self.kind {
            EnvironmentRecordKind::Declarative => {
                let has_binding = self.bindings.contains_key(&name);

                Ok(CompletionRecordNormal(has_binding))
            }
            _ => todo!(),
        }
    }

    pub fn create_mutable_binding(
        &mut self,
        name: JsString,
        deletable: bool,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED>> {
        match self.kind {
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
            _ => todo!(),
        }
    }

    pub fn create_immutable_binding(
        &mut self,
        name: JsString,
        strict: bool,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED>> {
        match self.kind {
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
            _ => todo!(),
        }
    }

    pub fn initialize_binding(
        &mut self,
        name: JsString,
        value: Value,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED>> {
        match self.kind {
            EnvironmentRecordKind::Declarative => {
                if !self.bindings.contains_key(&name) {
                    return Err(CompletionRecordThrow(()));
                }

                let binding = self.bindings.get_mut(&name).unwrap();
                if binding.initialized {
                    return Err(CompletionRecordThrow(()));
                }

                binding.value = value;
                binding.initialized = true;

                Ok(CompletionRecordNormal(()))
            }
            _ => todo!(),
        }
    }

    pub fn set_mutable_binding(
        &mut self,
        name: JsString,
        value: Value,
        strict: bool,
    ) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED>> {
        match self.kind {
            EnvironmentRecordKind::Declarative => {
                if !self.bindings.contains_key(&name) {
                    return Err(CompletionRecordThrow(()));
                }

                let binding = self.bindings.get_mut(&name).unwrap();
                if (strict && !binding.mutable) || binding.initialized {
                    return Err(CompletionRecordThrow(()));
                }

                binding.value = value;

                Ok(CompletionRecordNormal(()))
            }
            _ => todo!(),
        }
    }

    pub fn get_binding_value(
        &self,
        name: JsString,
        strict: bool,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError>> {
        match self.kind {
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
            _ => todo!(),
        }
    }

    pub fn delete_binding(
        &mut self,
        name: JsString,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<UNUSED>> {
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

    // TODO: Implement other methods (this, super based methods)
}

pub fn get_identifier_reference(
    name: JsString,
    env: Option<EnvironmentRecord>,
    strict: bool,
) -> Result<CompletionRecord<Reference>, CompletionRecord<()>> {
    match env {
        None => Ok(CompletionRecordNormal(Reference {
            base: ReferenceBase::Unresolvable,
            referenced_name: Value::String(name),
            strict,
            this_value: None,
        })),
        Some(env) => {
            let exists = env.has_binding(name.clone())?;
            if exists.value {
                Ok(CompletionRecordNormal(Reference {
                    base: ReferenceBase::EnvironmentRecord(env),
                    referenced_name: Value::String(name),
                    strict,
                    this_value: None,
                }))
            } else {
                get_identifier_reference(
                    name,
                    env.outer_env.clone().map(|rc| (*rc).clone()),
                    strict,
                )
            }
        }
    }
}
