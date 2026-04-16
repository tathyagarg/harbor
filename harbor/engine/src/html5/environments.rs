use std::{cell::RefCell, rc::Rc};

use crate::{
    http::url::URL,
    js::executable::{
        context::{ExecutionContext, pop_execution_context, push_execution_context},
        realm::Realm,
    },
};

pub struct Environment {
    pub id: String,
    pub creation_url: URL,

    pub top_level_creation_url: Option<URL>,

    pub execution_ready_flag: bool,
}

#[derive(Clone, Debug)]
pub struct EnvironmentSettings {
    pub realm: Rc<RefCell<Realm>>,
    pub realm_execution_context: Rc<RefCell<ExecutionContext>>,
}

impl EnvironmentSettings {
    pub fn prepare_to_run(&mut self) {
        push_execution_context(self.realm_execution_context.clone());
    }

    pub fn cleanup_after_running(&mut self) {
        pop_execution_context();
    }
}

impl PartialEq for EnvironmentSettings {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(
            &self.realm_execution_context,
            &other.realm_execution_context,
        )
    }
}

impl Eq for EnvironmentSettings {}

#[derive(Debug, Clone)]
pub struct HostDefined {
    pub settings: EnvironmentSettings,
}
