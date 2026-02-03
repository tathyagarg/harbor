use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use winit::event_loop::EventLoop;

use crate::{
    css::{cssom::CSSStyleSheet, layout::Layout, parser::parse_stylesheet, tokenize::tokenize},
    globals::{INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH},
    html5::{dom::Document, parse::Parser},
    http::{Client, Protocol, Request, url::URL},
    infra::{InputStream, Serializable},
    render::{App, WindowOptions},
};

pub struct Agent {
    cached_pages: HashMap<String, Rc<RefCell<Document>>>,

    ua_stylesheet: CSSStyleSheet,

    app: Option<App>,

    http_client: Client,
}

impl Agent {
    pub fn new() -> Self {
        let app = App {
            window_options: WindowOptions {
                use_transparent: true,
                background_color: wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.0,
                },
            },
            state: None,
            document: None,
            layout: None,
        };

        let client = Client::new(Protocol::HTTP1_1, true);

        let stylesheet_content = fs::read_to_string("res/css/ua.css").unwrap();

        let css_content = parse_stylesheet(
            &mut InputStream::new(&tokenize(&mut InputStream::new(
                &stylesheet_content.chars().collect::<Vec<char>>()[..],
            ))),
            None,
            None,
        );

        Self {
            cached_pages: HashMap::new(),
            app: Some(app),
            http_client: client,
            ua_stylesheet: css_content,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        self.open("https://old.arson.dev/");

        let _ = event_loop.run_app(self.app.as_mut().unwrap());
    }

    pub fn open(&mut self, url: &str) {
        let maybe_resolved_url = URL::pure_parse(url.to_string());

        if maybe_resolved_url.is_err() {
            return;
        }

        let resolved_url = maybe_resolved_url.unwrap().serialize();

        if let Some(doc) = self.cached_pages.get(&resolved_url) {
            let document = doc.clone();

            if let Some(app) = &mut self.app {
                app.document = Some(document);
            }
        } else {
            let url_obj = self.http_client.connect_to_url(resolved_url);

            let _response = self.http_client.send_request(Request {
                method: String::from("GET"),
                request_target: url_obj.path.serialize(),
                protocol: Protocol::HTTP1_1,
                headers: vec![
                    crate::http::Header::new(
                        String::from("User-Agent"),
                        String::from("Harbor Browser"),
                    ),
                    crate::http::Header::new(
                        String::from("Host"),
                        url_obj.host.unwrap().serialize(),
                    ),
                ],
                body: None,
            });

            let response = match _response {
                Some(resp) => resp,
                None => return,
            };

            println!(
                "Received response: \n\n{}",
                response.body.clone().unwrap_or_default()
            );

            let html_content = match response.body {
                Some(body) => body,
                None => return,
            };

            let mut stream = InputStream::new(&html_content.chars().collect::<Vec<char>>()[..]);

            let document = Parser::parse_stream(&mut stream);
            document
                .borrow_mut()
                .insert_stylesheet(0, self.ua_stylesheet.clone());

            let mut layout = Layout::new(
                Rc::clone(&document),
                (INITIAL_WINDOW_WIDTH as f64, INITIAL_WINDOW_HEIGHT as f64),
            );

            layout.make_tree();
            layout.layout();

            if let Some(app) = &mut self.app {
                app.document = Some(Rc::clone(&document));
                app.layout = Some(layout);
            }
        }
    }
}
