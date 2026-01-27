use std::rc::Rc;

use crate::{
    css::{layout::Layout, tokenize::tokenize},
    infra::{InputStream, Serializable},
};

pub mod css;
pub mod font;
pub mod globals;
pub mod html5;
pub mod http;
pub mod infra;
pub mod render;

use crate::css::parser::parse_stylesheet;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let url_target = String::from("https://flavorless.hackclub.com/");
    println!("Parsing target: {}", url_target);

    let mut client = http::Client::new(http::Protocol::HTTP1_1, true);
    let url = client.connect_to_url(url_target);

    println!("Sending request to: {}", url.serialize());

    let resp = client.send_request(http::Request {
        method: String::from("GET"),
        request_target: url.path.serialize(),
        protocol: http::Protocol::HTTP1_1,
        headers: vec![
            http::Header::new(String::from("User-Agent"), String::from("Harbor Browser")),
            http::Header::new(String::from("Host"), url.host.unwrap().serialize()),
        ],
        body: None,
    });

    let response = resp.unwrap();
    println!("Received response: \n\n{}", response.body.clone().unwrap());

    let html_content = response.body.unwrap();
    // let html_content = include_str!("../../assets/html/custom002.html");

    let mut stream = InputStream::new(&html_content.chars().collect::<Vec<char>>()[..]);
    let mut parser = html5::parse::Parser::new(&mut stream);

    parser.parse();

    let stylesheet = include_str!("../../assets/css/ua.css").to_string();
    let css_content = parse_stylesheet(
        &mut InputStream::new(&tokenize(&mut InputStream::new(
            &stylesheet.chars().collect::<Vec<char>>()[..],
        ))),
        Rc::downgrade(parser.document.document()),
        None,
    );

    parser
        .document
        .document()
        .borrow_mut()
        .insert_stylesheet(0, css_content);

    let mut layout = Layout::new(Rc::clone(&parser.document.document()), (800.0, 600.0));
    layout.make_tree();
    layout.layout();

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
        state: None,
        document: parser.document.document.borrow().clone(),
        layout,
    };

    _ = event_loop.run_app(&mut app);
}
