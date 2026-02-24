use crate::{
    http::url::{self, special_scheme_default_port},
    infra::Serializable,
};

pub mod anchor;
pub mod link;

pub struct HyperlinkUtils {
    pub href: String,
    pub origin: String,
    pub protocol: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
}

impl HyperlinkUtils {
    pub fn new(raw_url: &str) -> Option<Self> {
        let url = url::URL::pure_parse(raw_url.to_string()).ok()?;

        Some(Self {
            href: url.serialize(),
            origin: url.origin().unwrap_or("".to_string()),
            protocol: url.scheme.clone(),
            username: url.username,
            password: url.password,
            host: url.host.clone().map_or(String::new(), |h| h.serialize()),
            hostname: url.host.map_or(String::new(), |h| h.serialize()),
            port: url.port.map_or(
                (special_scheme_default_port(&url.scheme).unwrap_or(0)).to_string(),
                |p| p.to_string(),
            ),
            pathname: url.path.serialize(),
            search: url.query.map_or("".to_string(), |q| format!("?{}", q)),
            hash: url.fragment.map_or("".to_string(), |f| format!("#{}", f)),
        })
    }
}
