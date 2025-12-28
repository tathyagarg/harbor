pub mod client;
pub mod dns;
pub mod url;

pub use client::*;

#[derive(Default, Debug, Clone)]
pub enum Scheme {
    #[default]
    HTTP,
    HTTPS,
}

#[derive(Default, Debug)]
pub struct URL {
    scheme: Scheme,
    pub host: String,
    port: Option<u16>,
    path: String,
}

impl URL {
    pub fn reconstruct(&self) -> String {
        let mut url = String::new();

        match self.scheme {
            Scheme::HTTP => url.push_str("http"),
            Scheme::HTTPS => url.push_str("https"),
        }

        url.push_str("://");

        url.push_str(&self.host);

        match self.scheme {
            Scheme::HTTP => {
                if self.port.is_some() && self.port.unwrap() != 80 {
                    url.push_str(&format!(":{}", self.port.unwrap()));
                }
            }
            Scheme::HTTPS => {
                if self.port.is_some() && self.port.unwrap() != 443 {
                    url.push_str(&format!(":{}", self.port.unwrap()));
                }
            }
        }

        url.push_str(&self.path);

        url
    }
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

            url_obj.port = Some(port.parse::<u16>().unwrap());
            url_obj.path = "/".to_owned() + path;
        } else {
            url_obj.port = Some(url.parse::<u16>().unwrap());
            url_obj.path = String::from("/");
        }
    } else {
        if url.contains("/") {
            let (host, path) = url.split_once("/").unwrap();

            url_obj.host = host.to_string();
            url_obj.path = "/".to_owned() + path;
        } else {
            url_obj.host = url;
            url_obj.path = String::from("/");
        }
    }

    Some(url_obj)
}

pub fn preferred_default_port(scheme: Scheme) -> u16 {
    match scheme {
        Scheme::HTTP => 80,
        Scheme::HTTPS => 443,
    }
}
