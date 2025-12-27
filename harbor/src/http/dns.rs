use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    time::Instant,
};

use crate::http;

pub struct DnsResolver {
    resolved_urls: HashMap<(String, u16), (SocketAddr, Instant)>,
}

pub const DEFAULT_TTL_SECS: u64 = 300;

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            resolved_urls: HashMap::new(),
        }
    }

    pub fn resolve_url(&mut self, url: String) -> SocketAddr {
        let url_obj = http::construct_url(url).unwrap();

        self.resolve(
            url_obj.host,
            url_obj
                .port
                .unwrap_or(http::preferred_default_port(url_obj.scheme)),
        )
    }

    pub fn resolve(&mut self, host: String, port: u16) -> SocketAddr {
        let pair = (host.clone(), port);

        if let Some((addr, created_at)) = self.resolved_urls.get(&pair) {
            if created_at.elapsed().as_secs() >= DEFAULT_TTL_SECS {
                self.resolved_urls.remove(&pair);
            } else {
                return *addr;
            }
        }

        let mut addrs = (host.as_str(), port).to_socket_addrs().unwrap();
        let sock_addr = addrs.next().unwrap();

        self.resolved_urls
            .insert((host, port), (sock_addr, Instant::now()));

        sock_addr
    }
}
