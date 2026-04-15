use std::{cell::RefCell, rc::Rc};

use crate::js::{
    executable::{
        agent::{SURROUNDING_AGENT, running_execution_context},
        environment::{EnvironmentRecord, get_identifier_reference},
        realm::Realm,
    },
    types::completion_record::{CRKThrow, CompletionRecord},
    values::{object::FunctionObject, reference::Reference, string::JsString},
};

#[derive(Clone, Debug)]
pub enum ScriptOrModule {
    Script,
    Module,
}

#[derive(Clone, Debug)]
pub enum ExecutionContext {
    Generic(GenericExecutionContext),
    Code(CodeExecutionContext),
}

impl ExecutionContext {
    pub fn script_or_module(&self) -> Option<ScriptOrModule> {
        match self {
            ExecutionContext::Generic(ctx) => ctx.script_or_module.clone(),
            ExecutionContext::Code(ctx) => ctx.execution_context.script_or_module.clone(),
        }
    }

    pub fn lexical_env(&self) -> Option<Rc<RefCell<EnvironmentRecord>>> {
        match self {
            ExecutionContext::Generic(_) => None,
            ExecutionContext::Code(ctx) => Some(ctx.lexical_env.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenericExecutionContext {
    pub function: Option<FunctionObject>,
    pub realm: Rc<RefCell<Realm>>,

    pub script_or_module: Option<ScriptOrModule>,
}

#[derive(Clone, Debug)]
pub struct CodeExecutionContext {
    pub execution_context: GenericExecutionContext,

    pub lexical_env: Rc<RefCell<EnvironmentRecord>>,
    pub variable_env: Rc<RefCell<EnvironmentRecord>>,
}

pub fn get_active_script_or_module() -> Option<ScriptOrModule> {
    SURROUNDING_AGENT.with(|agent| {
        if let Some(agent) = agent.borrow().as_ref() {
            let agent_borrow = agent.borrow();
            if agent_borrow.execution_context_stack.is_empty() {
                return None;
            }

            let ec = agent_borrow
                .execution_context_stack
                .iter()
                .rfind(|context| context.script_or_module().is_some());

            ec.and_then(|context| context.script_or_module().clone())
        } else {
            panic!("No surrounding agent found");
        }
    })
}

pub fn resolve_binding(
    name: JsString,
    env: Option<EnvironmentRecord>,
) -> Result<CompletionRecord<Reference>, CompletionRecord<(), CRKThrow>> {
    let ctx = running_execution_context().unwrap();

    let env = env.unwrap_or_else(|| ctx.lexical_env().unwrap().borrow().clone());
    let strict = true;

    return get_identifier_reference(name, Some(env), strict);
}

pub fn push_execution_context(context: Rc<ExecutionContext>) {
    SURROUNDING_AGENT.with(|agent| {
        if let Some(agent) = agent.borrow_mut().as_mut() {
            agent.borrow_mut().execution_context_stack.push(context);
        } else {
            panic!("No surrounding agent found");
        }
    })
}
