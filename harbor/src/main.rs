mod font;
mod html5;
mod http;
mod render;

use winit::event_loop::EventLoop;

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

    // let event_loop = EventLoop::with_user_event().build().unwrap();
    // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // let mut app = render::App {
    //     window_options: render::WindowOptions {
    //         use_transparent: true,
    //         background_color: wgpu::Color {
    //             r: 0.2,
    //             g: 0.8,
    //             b: 0.2,
    //             a: 0.0,
    //         },
    //     },
    //     ..Default::default()
    // };
    // _ = event_loop.run_app(&mut app);

    let ttc_data = include_bytes!("../../assets/fonts/Times.ttc");

    println!("{:#?}", font::parse_ttc(ttc_data));
}
