use std::{collections::HashMap, ops::Deref, rc::Rc, sync::LazyLock};

use crate::{
    css::{layout::Layout, parser::preprocess, tokenize::tokenize},
    infra::InputStream,
};

pub mod css;
pub mod font;
pub mod globals;
pub mod html5;
pub mod http;
pub mod infra;
pub mod render;

use crate::css::parser::parse_stylesheet;
use crate::font::ttf::ParsedTableDirectory;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let html_content = include_str!("../../assets/html/custom001.html");
    let html_chars = html_content.chars().collect::<Vec<char>>();
    let html_slice = &html_chars[..];

    let mut stream = InputStream::new(html_slice);
    let mut parser = html5::parse::Parser::new(&mut stream);

    parser.parse();

    // let mut tokenizer = css::parser::CSSParser::new(css_content);

    // tokenizer.tokenize();

    // println!("CSS Tokens: {:#?}", tokenizer.tokens);

    // let url_target = String::from("https://old.arson.dev/");
    // println!("Parsing target: {}", url_target);

    // let mut client = http::Client::new(http::Protocol::HTTP1_1, true);
    // let url = client.connect_to_url(url_target);

    // println!("Sending request to: {}", url.serialize());

    // let resp = client.send_request(http::Request {
    //     method: String::from("GET"),
    //     request_target: url.path.serialize(),
    //     protocol: http::Protocol::HTTP1_1,
    //     headers: vec![
    //         http::Header::new(String::from("User-Agent"), String::from("Harbor Browser")),
    //         http::Header::new(String::from("Host"), url.host.unwrap().serialize()),
    //     ],
    //     body: None,
    // });

    // let response = resp.unwrap();
    // println!("Received response: \n\n{}", response.body.clone().unwrap());

    // let html_content = include_str!("../../assets/html/css002.html");

    // let mut stream = html5::parse::InputStream::new(html_content.to_string());
    // let mut tokenizer = html5::parse::Parser::new(&mut stream);

    // tokenizer.tokenize();

    let mut layout = Layout::new(Rc::clone(&parser.document.document()), (800.0, 600.0));
    layout.make_tree();
    layout.layout();

    // println!("Layout Tree: {:#?}", layout.root_box);

    // layout.register_font("Times New Roman", times.complete());
    // layout.register_font("FiraCode", fira_code.complete());
    // layout.register_font("SFNS", sfns.complete());

    let event_loop = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = render::App {
        window_options: render::WindowOptions {
            use_transparent: true,
            background_color: wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            },
        },
        // font: ttf.clone(),
        // text: "Hello, world!".to_string(),
        // vertices: vec![],
        state: None,
        document: parser.document.document.borrow().clone(),
        layout,
    };

    _ = event_loop.run_app(&mut app);
}
