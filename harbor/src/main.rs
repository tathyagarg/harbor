mod font;
mod html5;
mod http;
mod render;

use winit::event_loop::EventLoop;

use crate::render::TextRenderer;

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

    // if let Some(response) = resp {
    //     println!("Received response: \n\n{}", response.body.clone().unwrap());

    //     let mut stream = html5::parse::InputStream::new(response.body.unwrap());
    //     let mut tokenizer = html5::parse::Parser::new(&mut stream);

    //     tokenizer.tokenize();

    //     println!("Document Tree:");
    //     let dom_length = format!("{:?}", tokenizer.document.document()._node).len();
    //     println!(
    //         "If printed, the DOM would be {} characters long.",
    //         dom_length
    //     );
    //     println!(
    //         "Extra dev note: I manually went through the DOM and can confirm it looks correct."
    //     );
    // }

    // let ttc_data = include_bytes!("../../assets/fonts/Times.ttc");
    // let ttc = font::parse_ttc(ttc_data);
    // let ttf = ttc.table_directories.first().unwrap().complete();

    let ttf_data = include_bytes!("../../assets/fonts/FiraCode.ttf");
    let ttf = font::parse_ttf(ttf_data);

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
        text_renderer: TextRenderer::new()
            .with_font(ttf.complete())
            .with_text(vec![
                (String::from("Hello, world!"), 160.0),
                (
                    String::from("This is a test of the Harbor browser font rendering system."),
                    48.0,
                ),
                (
                    String::from("The quick brown fox jumps over the lazy dog."),
                    72.0,
                ),
                (String::from("Made with <3 by Tathya"), 36.0),
            ])
            .with_origin((10.0, 10.0)),
        // resize_function: Box::new(resize_function),
    };
    _ = event_loop.run_app(&mut app);
}
