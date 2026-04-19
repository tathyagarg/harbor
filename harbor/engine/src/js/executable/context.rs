use std::{cell::RefCell, rc::Rc};

use crate::js::{
    r#abstract::Generator,
    executable::{
        agent::{SURROUNDING_AGENT, running_execution_context},
        environment::{EnvironmentRecord, get_identifier_reference},
        realm::Realm,
    },
    script::ScriptRecord,
    types::completion_record::{CRKThrow, CompletionRecord},
    values::{object::FunctionObject, reference::Reference, string::JsString},
};

#[derive(Clone, Debug)]
pub enum ScriptOrModule {
    Script(Rc<ScriptRecord>),
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

    pub fn replace_lexical_env(&mut self, new_env: Rc<RefCell<EnvironmentRecord>>) {
        if let ExecutionContext::Code(ctx) = self {
            ctx.lexical_env = new_env;
        } else {
            panic!("replace_lexical_env called on a non-code execution context");
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenericExecutionContext {
    pub function: Option<FunctionObject>,
    pub generator: Option<Rc<RefCell<Generator>>>,
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
                .rfind(|context| context.borrow().script_or_module().is_some());

            ec.and_then(|context| context.borrow().script_or_module().clone())
        } else {
            panic!("No surrounding agent found");
        }
    })
}

pub fn resolve_binding(
    name: JsString,
    env: Option<Rc<RefCell<EnvironmentRecord>>>,
) -> Result<CompletionRecord<Reference>, CompletionRecord<(), CRKThrow>> {
    let ctx = running_execution_context().unwrap();

    let env = env.unwrap_or_else(|| ctx.borrow().lexical_env().unwrap());
    let strict = true;

    return get_identifier_reference(name, Some(env), strict);
}

pub fn push_execution_context(context: Rc<RefCell<ExecutionContext>>) {
    SURROUNDING_AGENT.with(|agent| {
        if let Some(agent) = agent.borrow_mut().as_mut() {
            agent.borrow_mut().execution_context_stack.push(context);
        } else {
            panic!("No surrounding agent found");
        }
    })
}

pub fn pop_execution_context() {
    SURROUNDING_AGENT.with(|agent| {
        if let Some(agent) = agent.borrow_mut().as_mut() {
            agent.borrow_mut().execution_context_stack.pop();
        } else {
            panic!("No surrounding agent found");
        }
    })
}
