use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};

use crate::js::collect_seq;
use crate::js::script::global_declaration_instantiation;
use crate::js::values::string::JsString;
use crate::js::{executable::realm::current_realm, script::parse_script};

use crate::js::{
    executable::{
        agent::{Agent, AgentRecord, SURROUNDING_AGENT},
        realm::initialize_host_defined_realm,
    },
    semantics::statements,
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

    let text = r#"const a = 1 + 2;
const b = a * 2;"#;
    println!("Running script:\n{}\n", text);

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

    let script = parse_script(text, current_realm());

    global_declaration_instantiation(
        &script.ecma_script_code,
        current_realm()
            .borrow()
            .global_env
            .as_ref()
            .unwrap()
            .clone(),
    );

    let slice = collect_seq(&script.ecma_script_code.body);

    for stmt in slice.iter() {
        statements::statement_or_declaration_evaluate(stmt);
    }

    let test = JsString::from_str("b").unwrap();
    let env = current_realm()
        .borrow()
        .global_env
        .as_ref()
        .unwrap()
        .clone();

    let res = env.borrow().get_binding_value(test, false).unwrap().value;
    println!("Value of b: {:?}", res);

    // let ua = Agent::new();
    // let mut app = App::new(
    //     render::WindowOptions {
    //         use_transparent: true,
    //         background_color: wgpu::Color {
    //             r: 1.0,
    //             g: 1.0,
    //             b: 1.0,
    //             a: 1.0,
    //         },
    //     },
    //     Some(Rc::clone(&ua)),
    // );

    // app.run();
}
