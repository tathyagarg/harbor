mod font;
mod html5;
mod http;
mod render;

use winit::event_loop::EventLoop;

use crate::font::{
    tables::glyf::{GlyphDataType, Point},
    ttf::TableRecordData,
};

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
    let font_size = 400.0;

    let scale = font_size / units_per_em;

    let glyph_index = cmap.char_to_glyph_index('A' as u32).unwrap() as usize;
    let glyph = glyf.glyphs.get(glyph_index).unwrap();

    let mut segments = Vec::<render::Segment>::new();
    let mut vertices = Vec::new();

    let origin = (20.0, 350.0);
    let color = [0.0, 0.0, 0.0];

    let width = 800.0;
    let height = 600.0;

    let vertex_maker = render::VertexMaker::new(origin, scale, width, height, color);

    match &glyph.data {
        GlyphDataType::Simple(simple) => {
            for contour in &simple.contours {
                // populate segments
                let mut contour_points = contour.points.clone();

                let mut i = 0;

                while i < contour_points.len() {
                    let current_point = &contour_points[i];
                    let next_point = &contour_points[(i + 1) % contour_points.len()];

                    if current_point.on_curve && next_point.on_curve {
                        // Line segment
                        segments.push(render::Segment::Line(
                            vertex_maker.from_point(current_point),
                            vertex_maker.from_point(next_point),
                        ));
                        i += 1;
                    } else if current_point.on_curve && !next_point.on_curve {
                        let next_next_point = &contour_points[(i + 2) % contour_points.len()];
                        if next_next_point.on_curve {
                            // Quadratic Bezier segment
                            segments.push(render::Segment::Quadratic(
                                vertex_maker.from_point(current_point),
                                vertex_maker.from_point(next_point),
                                vertex_maker.from_point(next_next_point),
                            ));
                            i += 2;
                        } else {
                            // Implied on-curve point
                            let implied_point = Point::midpoint(next_point, next_next_point);

                            segments.push(render::Segment::Quadratic(
                                vertex_maker.from_point(current_point),
                                vertex_maker.from_point(next_point),
                                vertex_maker.from_point(&implied_point),
                            ));
                            contour_points.insert((i + 2) % contour_points.len(), implied_point);

                            i += 2;
                        }
                    } else {
                        // This case should not happen in well-formed glyphs
                        println!("Warning: Two consecutive off-curve points found.");
                        i += 1;
                    }
                }
            }
        }
        GlyphDataType::Composite(composite) => {
            println!("Rendering composite glyph...");
        }
    }

    // panic!();

    for segment in &segments {
        segment.flatten(&mut vertices, scale / 20.0);
    }

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
