use std::{cell::RefCell, rc::Rc};

use crate::{
    globals::ENABLE_JS,
    js::executable::{
        agent::{Agent, AgentRecord, SURROUNDING_AGENT},
        realm::initialize_host_defined_realm,
    },
};

use crate::{render::App, user_agent::Agent as UAgent};

pub mod css;
pub mod font;
pub mod globals;
pub mod html5;
pub mod http;
pub mod infra;
pub mod js;
pub mod render;
pub mod user_agent;

fn main() {
    env_logger::init();

    unsafe {
        ENABLE_JS = std::env::args().any(|arg| arg == "--enable-js");
    }

    SURROUNDING_AGENT.with(|cell| {
        *cell.borrow_mut() = Some(Rc::new(RefCell::new(Agent {
            execution_context_stack: Vec::new(),
            record: AgentRecord {
                little_endian: cfg!(target_endian = "little"),
                can_block: true,
                signifier: 0, // TODO: generate unique signifiers
                is_lock_free_1: true,
                is_lock_free_2: true,
                is_lock_free_8: true,
                candidate_execution: (),
                kept_alive: Vec::new(),
                module_async_evaluation_count: 0,
            },
            executing_thread: (),
        })));

        initialize_host_defined_realm().unwrap();
    });

    let ua = UAgent::new();
    let mut app = App::new(
        render::WindowOptions {
            use_transparent: true,
            background_color: wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        },
        Some(Rc::clone(&ua)),
    );

    app.run();
}
