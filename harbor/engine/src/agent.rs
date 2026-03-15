use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use crate::{
    css::{cssom::CSSStyleSheet, parser::parse_stylesheet, tokenize::tokenize},
    globals::{
        ERROR, ERROR_PAGE_PATH, NEW_TAB, NEW_TAB_PAGE_PATH, NO_CONNECTION, NO_CONNECTION_PAGE_PATH,
        NO_CONNECTION_URL, RES_PATH,
    },
    html5::{dom::Document, parse::Parser},
    http::{
        Client, Header, Protocol, Request, RequestIntegrityError, RequestIntegrityErrorKind,
        url::URL,
    },
    infra::{InputStream, Serializable},
};

pub struct Agent {
    cached_pages: HashMap<String, Rc<RefCell<Document>>>,

    ua_stylesheet: CSSStyleSheet,

    http_client: Client,
}

impl Agent {
    pub fn new() -> Rc<RefCell<Self>> {
        let client = Client::new(Protocol::HTTP1_1, true);

        let ua_path = RES_PATH().join("css").join("ua.css");

        let stylesheet_content = fs::read_to_string(ua_path).unwrap();
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

    pub fn fetch_stylesheet(&mut self, url: &str) -> Option<CSSStyleSheet> {
        let maybe_resolved_url = URL::pure_parse(url.to_string());

        if maybe_resolved_url.is_err() {
            return None;
        }

        let raw_url = maybe_resolved_url.as_ref().unwrap();
        let resolved_url = raw_url.serialize();

        let body = if raw_url.scheme == "http" || raw_url.scheme == "https" {
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
                Ok(resp) => resp,
                Err(_) => {
                    eprintln!("Failed to fetch stylesheet URL: {}", resolved_url);
                    return None;
                }
            };

            match response.body {
                Some(body) => body,
                None => return None,
            }
        } else if raw_url.scheme == "file" {
            let temp_path = raw_url.path.serialize_as_fs_path();

            let path = if cfg!(target_os = "windows") {
                temp_path.trim_end_matches('\\')
            } else {
                temp_path.trim_end_matches('/')
            }
            .to_string();

            println!("Fetching stylesheet from file: {} ({:?})", path, raw_url);

            fs::read_to_string(path).unwrap()
        } else {
            return None;
        };
        let css_content = parse_stylesheet(
            &mut InputStream::new(&tokenize(&mut InputStream::new(
                &body.chars().collect::<Vec<char>>()[..],
            ))),
            None,
            None,
        );

        return Some(css_content);
    }

    fn open_error_page(&mut self, error: RequestIntegrityError) -> Rc<RefCell<Document>> {
        let url = match error.kind {
            RequestIntegrityErrorKind::NoConnection => NO_CONNECTION_URL,
            RequestIntegrityErrorKind::InvalidMethod => "harbor:invalidmethod",
            RequestIntegrityErrorKind::InvalidHeaders => "harbor:invalidheaders",
            RequestIntegrityErrorKind::InvalidBody => "harbor:invalidbody",
        };

        self.open(url).unwrap()
    }

    pub fn open(&mut self, url: &str) -> Option<Rc<RefCell<Document>>> {
        if url.starts_with("harbor:") {
            let path = url.trim_start_matches("harbor:").to_string();

            let path = match path.as_str() {
                NEW_TAB => NEW_TAB_PAGE_PATH,
                NO_CONNECTION => NO_CONNECTION_PAGE_PATH,
                ERROR => ERROR_PAGE_PATH,
                _ => ERROR_PAGE_PATH,
            }();

            println!("Opening internal page: {} ({:?})", url, path);

            let html_content = fs::read_to_string(&path).unwrap();
            let mut stream = InputStream::new(&html_content.chars().collect::<Vec<char>>()[..]);

            let document = Parser::parse_stream(&mut stream);
            document
                .borrow_mut()
                .insert_stylesheet(0, self.ua_stylesheet.clone());

            let this_url = format!(
                "file://{}",
                std::env::current_dir()
                    .unwrap()
                    .join(path)
                    .to_str()
                    .unwrap()
            );

            self.handle_link_elements(&this_url, &document);

            return Some(document);
        }

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
                    Ok(resp) => resp,
                    Err(e) => return Some(self.open_error_page(e)),
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

            self.handle_link_elements(url, &document);

            self.cached_pages.insert(resolved_url, Rc::clone(&document));

            return Some(document);
        }
    }

    fn handle_link_elements(&mut self, root_url: &str, document: &Rc<RefCell<Document>>) {
        let links = document.borrow().get_links();

        for link in links {
            if !link
                .borrow()
                .rel_list()
                .contains(&String::from("stylesheet"))
            {
                continue;
            }

            let joint = URL::join_urls(root_url.to_string(), link.borrow().href.clone()).unwrap();

            if let Some(stylesheet) = self.fetch_stylesheet(joint.serialize().as_str()) {
                document
                    .borrow_mut()
                    .insert_stylesheet(0, stylesheet.clone());
            }
        }
    }
}
