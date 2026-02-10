use std::rc::Rc;

use harbor::js::PunctuatorKind;

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

    let data = "// abc\nabc.def".encode_utf16().collect::<Vec<u16>>();

    println!(
        "========ORIGINAL STRING START=======\n{}\n========ORIGINAL STRING END=========",
        String::from_utf16_lossy(&data)
    );

    let string: js::ZigString = js::ZigString {
        data: data.as_ptr(),
        len: data.len(),
    };
    let tokens = unsafe { js::parse_text_string(string, 4) };

    for token in unsafe { std::slice::from_raw_parts(tokens.data, tokens.len) } {
        print!("Token: kind={:?}", token.kind);
        if token.kind == js::TokenKind::CommonToken {
            // treat value as a pointer to js::CommonTokenData
            let common_token_data = unsafe { *(token.value as *const js::CommonTokenData) };
            print!(", common_kind={:?}", common_token_data.common_token_kind);

            if common_token_data.common_token_kind == js::CommonTokenKind::IdentifierName {
                // treat value as a pointer to js::IdentifierNameTokenData
                let identifier_name_token_data =
                    unsafe { *(common_token_data.value as *const js::IdentifierNameTokenData) };

                let name = unsafe {
                    std::slice::from_raw_parts(
                        identifier_name_token_data.name.data,
                        identifier_name_token_data.name.len,
                    )
                };
                let name_string = String::from_utf16_lossy(name);
                println!(", name={}", name_string);
            } else if common_token_data.common_token_kind == js::CommonTokenKind::Punctuator {
                let punctuator = unsafe {
                    std::mem::transmute::<u8, js::PunctuatorKind>(common_token_data.value as u8)
                };

                println!(", punctuator={:?}", punctuator);
            } else {
                println!();
            }
        } else {
            println!();
        }
    }

    unsafe { js::free_token_seq(tokens) };

    // js::JsRuntime::new();

    // let ua = Agent::new();
    // let mut app = App {
    //     window_options: render::WindowOptions {
    //         use_transparent: true,
    //         background_color: wgpu::Color {
    //             r: 1.0,
    //             g: 1.0,
    //             b: 1.0,
    //             a: 0.0,
    //         },
    //     },
    //     state: None,
    //     document: None,
    //     agent: Some(Rc::clone(&ua)),
    //     callbacks: None,
    // };

    // app.run();

    // let url_target = String::from("https://rupnil.codes/");
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

    // let html_content = response.body.unwrap();
    // let html_content = include_str!("../../assets/html/custom003.html");

    // let mut stream = InputStream::new(&html_content.chars().collect::<Vec<char>>()[..]);
    // let mut parser = html5::parse::Parser::new(&mut stream);

    // parser.parse();

    // let stylesheet = include_str!("../res/css/ua.css").to_string();
    // let css_content = parse_stylesheet(
    //     &mut InputStream::new(&tokenize(&mut InputStream::new(
    //         &stylesheet.chars().collect::<Vec<char>>()[..],
    //     ))),
    //     Rc::downgrade(parser.document.document()),
    //     None,
    // );

    // parser
    //     .document
    //     .document()
    //     .borrow_mut()
    //     .insert_stylesheet(0, css_content);

    // let mut layout = Layout::new(
    //     Rc::clone(&parser.document.document()),
    //     (INITIAL_WINDOW_WIDTH as f64, INITIAL_WINDOW_HEIGHT as f64),
    // );
    // layout.make_tree();
    // layout.layout();

    // let event_loop = EventLoop::with_user_event().build().unwrap();
    // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // let mut app = render::App {
    //     window_options: render::WindowOptions {
    //         use_transparent: true,
    //         background_color: wgpu::Color {
    //             r: 1.0,
    //             g: 1.0,
    //             b: 1.0,
    //             a: 0.0,
    //         },
    //     },
    //     state: None,
    //     document: parser.document.document.borrow().clone(),
    //     layout,
    // };

    // _ = event_loop.run_app(&mut app);
}
