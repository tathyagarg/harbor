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
    render::{App, CallbackData, WindowOptions},
};

pub struct Agent {
    cached_pages: HashMap<String, Rc<RefCell<Document>>>,

    ua_stylesheet: CSSStyleSheet,

    http_client: Client,
}

impl Agent {
    pub fn new() -> Rc<RefCell<Self>> {
        let client = Client::new(Protocol::HTTP1_1, true);

        let stylesheet_content = fs::read_to_string("res/css/ua.css").unwrap();
        let css_content = parse_stylesheet(
            &mut InputStream::new(&tokenize(&mut InputStream::new(
                &stylesheet_content.chars().collect::<Vec<char>>()[..],
            ))),
            None,
            None,
        );

        let agent = Rc::new(RefCell::new(Self {
            cached_pages: HashMap::new(),
            http_client: client,
            ua_stylesheet: css_content,
        }));

        agent
    }

    pub fn trigger(&self) {
        println!("Agent triggered");
    }

    pub fn run(self_rc: Rc<RefCell<Self>>) {
        // let event_loop = EventLoop::with_user_event().build().unwrap();
        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        {
            let mut this = self_rc.borrow_mut();
            this.open("https://flavorless.hackclub.com/");
        }

        // let mut app = {
        //     let mut this = self_rc.borrow_mut();
        //     this.app.take().unwrap()
        // };

        // let _ = event_loop.run_app(&mut app);
    }

    pub fn open(&mut self, url: &str) -> Option<Rc<RefCell<Document>>> {
        let maybe_resolved_url = URL::pure_parse(url.to_string());

        if maybe_resolved_url.is_err() {
            return None;
        }

        let resolved_url = maybe_resolved_url.unwrap().serialize();

        if let Some(doc) = self.cached_pages.get(&resolved_url) {
            return Some(doc.clone());

            // if let Some(app) = &mut self.app {
            //     app.document = Some(document);
            // }
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
                None => return None,
            };

            println!(
                "Received response: \n\n{}",
                response.body.clone().unwrap_or_default()
            );

            let html_content = match response.body {
                Some(body) => body,
                None => return None,
            };

            let mut stream = InputStream::new(&html_content.chars().collect::<Vec<char>>()[..]);

            let document = Parser::parse_stream(&mut stream);
            document
                .borrow_mut()
                .insert_stylesheet(0, self.ua_stylesheet.clone());

            return Some(Rc::clone(&document));

            // if let Some(app) = &mut self.app {
            //     app.document = Some(Rc::clone(&document));
            //     app.layout = Some(layout);

            //     if let Some(window) = &app.state {
            //         println!("\n\n\n\n\n\nRequesting redraw for new page\n\n\n\n\n\n");
            //         window.window.request_redraw();
            //     }
            // } else {
            //     println!("App not initialized in Agent");
            // }
        }
    }
}
