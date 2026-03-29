use crate::js::{
    executable::{
        agent::{SURROUNDING_AGENT, running_execution_context},
        environment::{EnvironmentRecord, get_identifier_reference},
        realm::Realm,
    },
    types::completion_record::{CRKThrow, CompletionRecord},
    values::{reference::Reference, string::JsString},
};

#[derive(Clone, Debug)]
pub enum ScriptOrModule {
    Script,
    Module,
}

#[derive(Clone, Debug)]
pub struct ExecutionContext {
    pub realm: Realm,
    pub script_or_module: Option<ScriptOrModule>,

    pub lexical_env: EnvironmentRecord,
    pub variable_env: EnvironmentRecord,

    // NOTE: Is this right?
    pub is_strict: bool,
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
                .rfind(|context| context.script_or_module.is_some());

            ec.and_then(|context| context.script_or_module.clone())
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

    let env = env.unwrap_or_else(|| ctx.lexical_env.clone());
    let strict = ctx.is_strict;

    return get_identifier_reference(name, Some(env), strict);
}
