use std::rc::Rc;

use crate::{js::semantics::expressions::EvaluateExpressionTag, render::App, user_agent::Agent};

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

    let text = r#"const a = 1 + 2 + 3;"#;
    let text_utf16: Vec<u16> = text.encode_utf16().collect();

    let zig_string = js::expr::ZigString {
        data: text_utf16.as_ptr(),
        len: text.len(),
    };

    unsafe {
        let script = js::parse_script(zig_string);

        let slice = std::slice::from_raw_parts(script.body.items, script.body.len);
        let bindings = (*(*slice[0].data.declaration).data.lex_decl).bindings;
        let bindings_slice = std::slice::from_raw_parts(bindings.items, bindings.len);
        let binding_val = (*bindings_slice[0].initializer).value.value;

        let evaluated = js::semantics::expressions::general_evaluate(
            &EvaluateExpressionTag::AssignmentExpression(binding_val),
        )
        .get_value()
        .unwrap()
        .value;

        println!("Evaluated value: {:?}", evaluated);

        js::free_string(zig_string);
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
