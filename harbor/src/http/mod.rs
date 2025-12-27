pub mod client;
pub mod dns;

#[derive(Default, Debug)]
pub enum Scheme {
    #[default]
    HTTP,
    HTTPS,
}

#[derive(Default, Debug)]
pub struct URL {
    scheme: Scheme,
    pub host: String,
    port: u16,
    path: String,
}

pub fn construct_url(url: String) -> Option<URL> {
    let mut url_obj = URL::default();

    let (scheme, remaining) = url.split_once("://").unwrap();

    url_obj.scheme = match scheme {
        "http" => Scheme::HTTP,
        "https" => Scheme::HTTPS,
        _ => return None,
    };

    let url = remaining.to_string();
    if url.contains(":") {
        let (host, remaining) = url.split_once(":").unwrap();

        url_obj.host = host.to_string();

        let url = remaining.to_string();
        if url.contains("/") {
            let (port, path) = url.split_once("/").unwrap();

            url_obj.port = port.parse::<u16>().unwrap();
            url_obj.path = "/".to_owned() + path;
        } else {
            url_obj.port = url.parse::<u16>().unwrap();
            url_obj.path = String::from("/");
        }
    }

    Some(url_obj)
}

pub use client::*;
