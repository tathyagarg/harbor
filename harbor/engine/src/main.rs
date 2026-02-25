use std::rc::Rc;

use crate::{agent::Agent, render::App};

pub mod agent;
pub mod css;
pub mod font;
pub mod globals;
pub mod html5;
pub mod http;
pub mod infra;
pub mod js;
pub mod render;

fn main() {
    env_logger::init();

    let ua = Agent::new();
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
