mod font;
mod html5;
mod http;
mod render;

use winit::event_loop::EventLoop;

use crate::font::{tables::glyf::GlyphDataType, ttf::TableRecordData};

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

    let ttc_data = include_bytes!("../../assets/fonts/Times.ttc");
    let ttc = font::parse_ttc(ttc_data);
    let ttf = ttc.table_directories.first().unwrap();

    let cmap = match ttf.get_table_record(b"cmap").unwrap().data() {
        TableRecordData::CMAP(cmap_table) => cmap_table,
        _ => {
            return;
        }
    };

    let head = match ttf.get_table_record(b"head").unwrap().data() {
        TableRecordData::Head(head_table) => head_table,
        _ => {
            return;
        }
    };

    let glyf = match ttf.get_table_record(b"glyf").unwrap().data() {
        TableRecordData::Glyf(glyf_table) => glyf_table,
        _ => {
            return;
        }
    };

    let units_per_em = head.units_per_em as f32;
    println!("Units per EM: {}", units_per_em);
    let font_size = 120.0;

    let scale = font_size / units_per_em;

    let glyph_index = cmap.char_to_glyph_index('A' as u32).unwrap() as usize;
    let glyph = glyf.glyphs.get(glyph_index).unwrap();

    let mut vertices = Vec::new();

    let origin = (20.0, 100.0);

    match &glyph.data {
        GlyphDataType::Simple(simple) => {
            for contour in &simple.contours {
                for i in 0..contour.length {
                    let (x, y) = contour.points[i];

                    // scale x and align to center
                    let scaled_x = origin.0 + x as f32 * scale;
                    let scaled_y = origin.1 - y as f32 * scale;

                    vertices.push(
                        render::Vertex {
                            position: [scaled_x, scaled_y, 0.0],
                            color: [1.0, 0.0, 0.0],
                        }
                        .to_clip(800.0, 600.0),
                    );
                }

                vertices.push(
                    render::Vertex {
                        position: [
                            origin.0 + contour.points[0].0 as f32 * scale,
                            origin.1 - contour.points[0].1 as f32 * scale,
                            0.0,
                        ],
                        color: [1.0, 0.0, 0.0],
                    }
                    .to_clip(800.0, 600.0),
                );
            }

            println!("{:#?}", vertices);
        }
        GlyphDataType::Composite(composite) => {
            println!("Rendering composite glyph...");
        }
    }

    // panic!();

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
        vertices: vertices,
        ..Default::default()
    };
    _ = event_loop.run_app(&mut app);
}
