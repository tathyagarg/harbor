use std::{cell::RefCell, rc::Rc};

use crate::js::executable::context::ExecutionContext;

thread_local! {
    pub static SURROUNDING_AGENT: RefCell<Option<Rc<RefCell<Agent>>>> = RefCell::new(None);
}

pub type AgentSignifier = u64;

pub struct Agent {
    pub execution_context_stack: Vec<Rc<ExecutionContext>>,

    pub record: AgentRecord,

    pub executing_thread: (),
}

pub struct AgentRecord {
    // NOTE: const
    pub little_endian: bool,
    pub can_block: bool,

    // NOTE: const
    pub signifier: AgentSignifier,

    /// NOTE: const
    pub is_lock_free_1: bool,
    /// NOTE: const
    pub is_lock_free_2: bool,
    /// NOTE: const
    pub is_lock_free_8: bool,

    pub candidate_execution: (),

    pub kept_alive: Vec<()>,

    pub module_async_evaluation_count: u64,
}

pub fn agent_signifier() -> AgentSignifier {
    SURROUNDING_AGENT.with_borrow(|agent| {
        if let Some(agent) = agent.as_ref() {
            agent.borrow().record.signifier
        } else {
            panic!("No surrounding agent found");
        }
    })
}

pub fn agent_can_suspend() -> bool {
    SURROUNDING_AGENT.with_borrow(|agent| {
        if let Some(agent) = agent.as_ref() {
            agent.borrow().record.can_block
        } else {
            panic!("No surrounding agent found");
        }
    })
}

pub fn increment_module_async_evaluation_count() {
    SURROUNDING_AGENT.with_borrow_mut(|agent| {
        if let Some(agent) = agent.as_mut() {
            agent.borrow_mut().record.module_async_evaluation_count += 1;
        } else {
            panic!("No surrounding agent found");
        }
    })
}

pub fn running_execution_context() -> Option<Rc<ExecutionContext>> {
    SURROUNDING_AGENT.with_borrow(|agent| {
        if let Some(agent) = agent.as_ref() {
            let agent_borrow = agent.borrow();
            agent_borrow.execution_context_stack.last().cloned()
        } else {
            panic!("No surrounding agent found");
        }
    })
}
