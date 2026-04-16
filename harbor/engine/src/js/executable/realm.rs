use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    html5::environments::EnvironmentSettings,
    js::{
        behaviours::ordinary_object_create,
        executable::{
            agent::{AgentSignifier, agent_signifier, running_execution_context},
            context::{
                CodeExecutionContext, ExecutionContext, GenericExecutionContext,
                push_execution_context,
            },
            environment::{EnvironmentRecord, new_global_environment},
        },
        types::completion_record::{CRKThrow, CompletionRecord, CompletionRecordNormal, UNUSED},
        values::object::{Object, OrdinaryObject},
    },
};

#[derive(Debug, Clone)]
pub struct Realm {
    pub agent_signifier: AgentSignifier,

    pub intrinsics: (),

    /// NOTE: This is optional only because it needs to be empty during initialization. It will be set to
    /// Some(Object) before any code is executed in the realm.
    pub global_object: Option<Rc<RefCell<Object>>>,

    /// NOTE: This is optional only because it needs to be empty during initialization. It will be set to
    /// Some(EnvironmentRecord) before any code is executed in the realm.
    pub global_env: Option<Rc<RefCell<EnvironmentRecord>>>,

    /// WARN: This likely will NOT be implemented!
    pub template_map: (),

    pub loaded_modules: Vec<()>,

    pub host_defined: EnvironmentSettings,
}

pub fn current_realm() -> Rc<RefCell<Realm>> {
    let ec = running_execution_context();
    if let Some(ec) = ec {
        match ec.borrow().deref() {
            ExecutionContext::Generic(generic) => generic.realm.clone(),
            ExecutionContext::Code(code) => code.execution_context.realm.clone(),
        }
    } else {
        panic!("No current execution context found");
    }
}

pub fn initialize_host_defined_realm()
-> Result<CompletionRecord<UNUSED>, CompletionRecord<(), CRKThrow>> {
    let realm = Rc::new(RefCell::new(Realm {
        intrinsics: (),
        agent_signifier: agent_signifier(),
        template_map: (),

        global_object: None,
        global_env: None,

        loaded_modules: Vec::new(),
        host_defined: EnvironmentSettings {
            realm_execution_context: running_execution_context().unwrap(),
        },
    }));

    let global = Rc::new(RefCell::new(ordinary_object_create(
        Some(Object::Ordinary(OrdinaryObject::prototype())),
        Vec::new(),
    )));

    realm.borrow_mut().global_object = Some(global.clone());
    realm.borrow_mut().global_env = Some(Rc::new(RefCell::new(new_global_environment(
        &global, &global,
    ))));

    let ec = Rc::new(RefCell::new(ExecutionContext::Code(CodeExecutionContext {
        execution_context: GenericExecutionContext {
            function: None,
            realm: realm.clone(),
            script_or_module: None,
        },
        lexical_env: realm.borrow().global_env.as_ref().unwrap().clone(),
        variable_env: realm.borrow().global_env.as_ref().unwrap().clone(),
    })));

    push_execution_context(ec);

    Ok(CompletionRecordNormal(()))
}
