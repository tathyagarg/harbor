use std::{collections::HashMap, ops::Deref, rc::Rc};

use crate::{css::layout::Layout, http::url::Serializable, render::TextEntry};

mod css;
mod font;
mod html5;
mod http;
mod render;

use winit::event_loop::EventLoop;

use crate::render::TextRenderer;

fn text_size_map(tag: &str) -> f32 {
    match tag {
        "h1" => 32.0,
        "h2" => 28.0,
        "h3" => 24.0,
        "h4" => 20.0,
        "h5" => 16.0,
        "h6" => 14.0,
        "p" => 12.0,
        "span" => 12.0,
        _ => 12.0,
    }
}

fn main() {
    env_logger::init();

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

    let html_content = include_str!("../../assets/html/css002.html");

    let mut stream = html5::parse::InputStream::new(html_content.to_string());
    let mut tokenizer = html5::parse::Parser::new(&mut stream);

    tokenizer.tokenize();

    let mut layout = Layout::new(Rc::clone(&tokenizer.document.document()), (800.0, 600.0));
    layout.make_tree();
    layout.layout();

    println!("Layout Tree: {:#?}", layout.root_box);

    // println!("Document Tree:");
    // let dom_length = format!("{:?}", tokenizer.document.document()._node).len();
    // println!(
    //     "If printed, the DOM would be {} characters long.",
    //     dom_length
    // );
    // println!("Extra dev note: I manually went through the DOM and can confirm it looks correct.");

    // let body = tokenizer.document.get_elements_by_tag_name("body");

    // let body_elem = body.first().unwrap();
    // let text_nodes = body_elem
    //     .borrow()
    //     ._node
    //     .borrow()
    //     .child_nodes()
    //     .filter(|nodekind| match nodekind.borrow().deref() {
    //         html5::dom::NodeKind::Element(el) => matches!(
    //             el.borrow().qualified_name().to_ascii_lowercase().as_str(),
    //             "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span"
    //         ),
    //         _ => false,
    //     });

    // let extracted_text_contents = text_nodes
    //     .iter()
    //     .map(|nodekind| {
    //         let nk_borrow = nodekind.borrow();

    //         let el = match nk_borrow.deref() {
    //             html5::dom::NodeKind::Element(el) => el,
    //             _ => panic!("Expected Element node"),
    //         };

    //         let tag_name = el.borrow().qualified_name().to_ascii_lowercase();

    //         (
    //             el.borrow()
    //                 ._node
    //                 .borrow()
    //                 .child_nodes()
    //                 .iter()
    //                 .filter_map(|child_node| match child_node.borrow().deref() {
    //                     html5::dom::NodeKind::Text(text_node) => {
    //                         Some(text_node.borrow().data().to_string())
    //                     }
    //                     _ => None,
    //                 })
    //                 .collect::<Vec<String>>()
    //                 .join(" "),
    //             text_size_map(tag_name.as_str()),
    //         )
    //     })
    //     .collect::<Vec<(String, f32)>>();

    // println!(
    //     "\n\nExtracted Text Content:\n\n{:?}",
    //     extracted_text_contents
    // );

    let fira_code = font::parse_ttf(include_bytes!("../../assets/fonts/FiraCode.ttf"));
    let times =
        &font::parse_ttc(include_bytes!("../../assets/fonts/Times.ttc")).table_directories[0];
    let sfns = font::parse_ttf(include_bytes!("../../assets/fonts/SFNS.ttf"));

    layout.register_font("Times New Roman", times.complete());
    layout.register_font("FiraCode", fira_code.complete());
    layout.register_font("SFNS", sfns.complete());

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
        layout,
    };
    _ = event_loop.run_app(&mut app);
}
