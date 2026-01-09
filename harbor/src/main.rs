mod font;
mod html5;
mod http;
mod render;

use winit::event_loop::EventLoop;

use crate::font::ttf::TableRecordData;

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

    let event_loop = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = render::App {
        window_options: render::WindowOptions {
            use_transparent: true,
            background_color: wgpu::Color {
                r: 0.651,
                g: 0.89,
                b: 0.631,
                a: 0.0,
            },
        },
        ..Default::default()
    };
    _ = event_loop.run_app(&mut app);

    // let ttc_data = include_bytes!("../../assets/fonts/Times.ttc");
    // let ttc = font::parse_ttc(ttc_data);
    // let ttf = ttc.table_directories.first().unwrap();

    // let _cmap = ttf.get_table_record(b"cmap").unwrap().data();
    // if let TableRecordData::CMAP(cmap_table) = _cmap {
    //     let idx = cmap_table.char_to_glyph_index('A' as u32).unwrap();

    //     let _glyf = ttf.get_table_record(b"glyf").unwrap().data();

    //     if let TableRecordData::Glyf(glyf_table) = _glyf {
    //         let glyph_data = glyf_table.glyphs[idx as usize].clone();
    //         println!("Glyph data for 'A': {:?}", glyph_data);
    //     }
    // }
}
