use std::{cell::RefCell, rc::Rc};

use crate::{http::url::URL, js::executable::context::ExecutionContext};

pub struct Environment {
    pub id: String,
    pub creation_url: URL,

    pub top_level_creation_url: Option<URL>,

    pub execution_ready_flag: bool,
}

#[derive(Clone, Debug)]
pub struct EnvironmentSettings {
    pub realm_execution_context: Rc<RefCell<ExecutionContext>>,
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
