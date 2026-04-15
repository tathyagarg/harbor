use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};

use crate::js::stmt::STATEMENT_OR_DECLARATION_DECLARATION;

use crate::js::collect_seq;

use crate::js::executable::agent::running_execution_context;
use crate::js::script::global_declaration_instantiation;
use crate::js::stmt::DECLARATION_LEXICAL_DECLARATION;
use crate::js::values::string::JsString;
use crate::js::{executable::realm::current_realm, script::parse_script};

use crate::js::{
    executable::{
        agent::{Agent, AgentRecord, SURROUNDING_AGENT},
        context::{CodeExecutionContext, ExecutionContext, GenericExecutionContext},
        realm::{Realm, initialize_host_defined_realm},
    },
    semantics::statements,
};

use crate::{
    js::{
        executable::environment::EnvironmentRecord, semantics::expressions::EvaluateExpressionTag,
    },
    render::App,
    user_agent::Agent as UAgent,
};

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

    let text = r#"const a = 1 + 2;"#;

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

    unsafe {
        let script = parse_script(text, current_realm());
        println!("Script: {}", script.ecma_script_code);

        global_declaration_instantiation(
            &script.ecma_script_code,
            current_realm()
                .borrow()
                .global_env
                .as_ref()
                .unwrap()
                .clone(),
        );

        println!("Script 2: {}", script.ecma_script_code);

        let slice = collect_seq(&script.ecma_script_code.body);

        for item in &slice {
            println!("Item: {}", item);
        }

        let stmt = *(*slice[0].data.declaration).data.lex_decl;
        statements::declarations::evaluate(&stmt);

        let test = JsString::from_str("a").unwrap();
        let env = current_realm()
            .borrow()
            .global_env
            .as_ref()
            .unwrap()
            .clone();

        let res = env.borrow().get_binding_value(test, false).unwrap().value;
        println!("Value of a: {:?}", res);

        // let bindings = (*(*slice[0].data.declaration).data.lex_decl).bindings;
        // let bindings_slice = std::slice::from_raw_parts(bindings.items, bindings.len);
        // let binding_val = (*bindings_slice[0].initializer).value.value;

        // let evaluated = js::semantics::expressions::expression_evaluate(
        //     &EvaluateExpressionTag::AssignmentExpression(binding_val),
        // )
        // .get_value()
        // .unwrap()
        // .value;

        // println!("Evaluated value: {:?}", evaluated);
    }

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
