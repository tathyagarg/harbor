use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    time::Instant,
};

use crate::{
    http::{self},
    infra::Serializable,
};

pub struct DnsResolver {
    resolved_urls: HashMap<(http::url::Host, u16), (SocketAddr, Instant)>,
}

pub const DEFAULT_TTL_SECS: u64 = 300;

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            resolved_urls: HashMap::new(),
        }
    }

    pub fn resolve_url(&mut self, url: String) -> SocketAddr {
        let url_obj = http::url::URL::parse(url, None, None).unwrap();

        self.resolve(
            url_obj.host.unwrap(),
            url_obj
                .port
                .unwrap_or(http::url::special_scheme_default_port(&url_obj.scheme).unwrap()),
        )
    }

    pub fn resolve(&mut self, host: http::url::Host, port: u16) -> SocketAddr {
        let pair = (host.clone(), port);

        if let Some((addr, created_at)) = self.resolved_urls.get(&pair) {
            if created_at.elapsed().as_secs() >= DEFAULT_TTL_SECS {
                self.resolved_urls.remove(&pair);
            } else {
                return *addr;
            }
        }

        let mut addrs = (host.serialize(), port).to_socket_addrs().unwrap();
        let sock_addr = addrs.next().unwrap();

        self.resolved_urls
            .insert((host, port), (sock_addr, Instant::now()));

        sock_addr
    }
}
