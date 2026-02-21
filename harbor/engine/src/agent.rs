use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use crate::{
    css::{cssom::CSSStyleSheet, parser::parse_stylesheet, tokenize::tokenize},
    html5::{dom::Document, parse::Parser},
    http::{Client, Header, Protocol, Request, url::URL},
    infra::{InputStream, Serializable},
};

pub struct Agent {
    cached_pages: HashMap<String, Rc<RefCell<Document>>>,

    ua_stylesheet: CSSStyleSheet,

    http_client: Client,
}

impl Agent {
    pub fn new(url: Option<String>) -> Rc<RefCell<Self>> {
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

    pub fn open(&mut self, url: &str) -> Option<Rc<RefCell<Document>>> {
        let maybe_resolved_url = URL::pure_parse(url.to_string());

        if maybe_resolved_url.is_err() {
            return None;
        }

        let raw_url = maybe_resolved_url.as_ref().unwrap();
        let resolved_url = raw_url.serialize();

        if let Some(doc) = self.cached_pages.get(&resolved_url) {
            return Some(doc.clone());
        } else {
            let html_content = if raw_url.scheme == "http" || raw_url.scheme == "https" {
                let url_obj = self.http_client.connect_to_url(resolved_url.clone());

                let _response = self.http_client.send_request(Request {
                    method: String::from("GET"),
                    request_target: url_obj.path.serialize(),
                    protocol: Protocol::HTTP1_1,
                    headers: vec![
                        Header::new(String::from("User-Agent"), String::from("Harbor Browser")),
                        Header::new(String::from("Host"), url_obj.host.unwrap().serialize()),
                    ],
                    body: None,
                });

                let response = match _response {
                    Some(resp) => resp,
                    None => return None,
                };

                match response.body {
                    Some(body) => body,
                    None => return None,
                }
            } else if raw_url.scheme == "file" {
                let path = raw_url.path.serialize().trim_end_matches('/').to_string();

                fs::read_to_string(path).unwrap()
            } else {
                return None;
            };

            let mut stream = InputStream::new(&html_content.chars().collect::<Vec<char>>()[..]);

            let document = Parser::parse_stream(&mut stream);
            document
                .borrow_mut()
                .insert_stylesheet(0, self.ua_stylesheet.clone());

            self.cached_pages.insert(resolved_url, Rc::clone(&document));

            return Some(document);
        }
    }
}
