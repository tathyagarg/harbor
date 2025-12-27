use std::net::{SocketAddr, ToSocketAddrs};

use crate::http;

pub fn resolve_url(url: String) -> SocketAddr {
    let url_obj = http::construct_url(url).unwrap();

    resolve(url_obj.host, url_obj.port)
}

pub fn resolve(host: String, port: u16) -> SocketAddr {
    let mut addrs = (host.as_str(), port).to_socket_addrs().unwrap();

    addrs.next().unwrap()
}
