use std::{collections::HashMap, rc::Rc};

use crate::js::types::completion_record::{
    CompletionRecord, CompletionRecordError, CompletionRecordNormal, CompletionRecordThrow, UNUSED,
};

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
type Value = String;

pub struct Binding {
    pub value: Value,

    pub deletable: bool,
    pub mutable: bool,
    pub strict: bool,
    pub initialized: bool,
}

pub struct EnvironmentRecord {
    /// NOTE: This is not a Option<Box<EnvironmentRecord>> because, according to the spec:
    /// > An Environment Record may serve as the outer environment for multiple inner Environment Records
    /// [^]: https://tc39.es/ecma262/#sec-environment-records
    /// Such a structure is not possible with Box, but Rc allows for multiple ownership.
    pub outer_env: Option<Rc<EnvironmentRecord>>,

    pub kind: EnvironmentRecordKind,

    pub bindings: HashMap<String, Binding>,
}

impl EnvironmentRecord {
    pub fn has_binding(
        &self,
        name: String,
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
        name: String,
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
                        // NOTE: Change to Value::empty() or similar
                        value: Value::new(),
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
        name: String,
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
                        // NOTE: Change to Value::empty() or similar
                        value: Value::new(),
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
        name: String,
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
        name: String,
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
        name: String,
        strict: bool,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError>> {
        match self.kind {
            EnvironmentRecordKind::Declarative => {
                if !self.bindings.contains_key(&name) {
                    if strict {
                        return Err(CompletionRecordThrow(
                            CompletionRecordError::ReferenceError(name),
                        ));
                    } else {
                        // NOTE: Change to Value::undefined() or similar
                        return Ok(CompletionRecordNormal(Value::new()));
                    }
                }

                let binding = self.bindings.get(&name).unwrap();
                if !binding.initialized {
                    return Err(CompletionRecordThrow(
                        CompletionRecordError::ReferenceError(name),
                    ));
                }

                Ok(CompletionRecordNormal(binding.value.clone()))
            }
            _ => todo!(),
        }
    }

    pub fn delete_binding(
        &mut self,
        name: String,
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
