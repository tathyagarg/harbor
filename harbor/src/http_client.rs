use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

trait ReqEncodable {
    fn encode(&self) -> String;
}

#[derive(Debug)]
enum RequestIntegrityErrorKind {
    InvalidMethod,
    InvalidHeaders,
    InvalidBody,
}

#[derive(Debug)]
struct RequestIntegrityError {
    pub kind: RequestIntegrityErrorKind,
    pub message: String,
}

impl fmt::Display for RequestIntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format_args!(
                "RequestIntegrityError: {:?}\nInfo: {}",
                self.kind, self.message
            )
        )
    }
}

/// The HTTP protocl to use to encode and send the request
/// I *would* leave this as string rather than force an enum,
/// but I decided that HTTP protocols, unlike methods, are too standardized
/// to allow customization - not to mention the impact that the HTTP protocol
/// has on the overall structure of the request itself as well as how it's sent
#[allow(dead_code)]
pub enum Protocol {
    HTTP0_9,
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

impl ReqEncodable for Protocol {
    fn encode(&self) -> String {
        String::from(match self {
            Self::HTTP0_9 => "",
            Self::HTTP1_0 => "HTTP/1.0",
            Self::HTTP1_1 => "HTTP/1.1",
            Self::HTTP2_0 => "HTTP/2.0",
            Self::HTTP3_0 => "HTTP/3.0",
        })
    }
}

pub struct Header {
    pub name: String,
    pub value: String,
}

impl ReqEncodable for Header {
    fn encode(&self) -> String {
        format!("{}: {}\n", self.name, self.value)
    }
}

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Messages#http_requests
pub struct Request {
    pub method: String,
    pub request_target: String,
    pub protocol: Protocol,

    pub headers: Vec<Header>,

    pub body: Option<String>,
}

impl ReqEncodable for Request {
    fn encode(&self) -> String {
        let mut request = String::new();
        request.push_str(
            format!(
                "{} {} {}\r\n",
                self.method,
                self.request_target,
                self.protocol.encode()
            )
            .as_str(),
        );

        for header in self.headers.iter() {
            request.push_str(header.encode().as_str());
        }

        if self.body.is_some() {
            let body = self.body.as_ref().unwrap();

            request.push_str("\r\n");
            request.push_str(body.as_str());
        }

        if request.ends_with("\r\n") {
            request = request.strip_suffix("\r\n").unwrap().to_string();
        }
        request.push_str("\r\n\r\n");

        request
    }
}

impl Request {
    fn ensure_integrity(&self) -> Result<(), RequestIntegrityError> {
        match self.protocol {
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Evolution_of_HTTP#http0.9_%E2%80%93_the_one-line_protocol
            Protocol::HTTP0_9 => {
                if self.method != "GET" {
                    return Err(RequestIntegrityError {
                        kind: RequestIntegrityErrorKind::InvalidMethod,
                        message: format!(
                            "Only allowed method in HTTP/0.9 is 'GET', not {}",
                            self.method
                        ),
                    });
                }

                if self.headers.len() != 0 {
                    return Err(RequestIntegrityError {
                        kind: RequestIntegrityErrorKind::InvalidHeaders,
                        message: format!(
                            "HTTP/0.9 must take 0 header, found {}",
                            self.headers.len()
                        ),
                    });
                }

                if self.body.is_some() {
                    return Err(RequestIntegrityError {
                        kind: RequestIntegrityErrorKind::InvalidBody,
                        message: format!(
                            "No request body allowed in HTTP/0.9, found '{}'",
                            self.body.as_ref().unwrap()
                        ),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn send(&self, addr: String) -> Result<Response, RequestIntegrityError> {
        let integrity = self.ensure_integrity();
        if integrity.is_err() {
            return Err(integrity.unwrap_err());
        }

        match self.protocol {
            Protocol::HTTP0_9 => {
                let mut stream = TcpStream::connect(addr).unwrap();
                _ = stream.write(self.encode().as_bytes());

                let mut response = Response::new(Protocol::HTTP0_9);

                loop {
                    let mut chunk: [u8; 512] = [0; 512];
                    let bytes_read = stream.read(&mut chunk).unwrap();

                    if bytes_read == 0 {
                        break;
                    }

                    response.decode_body_chunk(&chunk);
                }

                Ok(response)
            }
            Protocol::HTTP1_0 => {
                let mut stream = TcpStream::connect(addr).unwrap();
                _ = stream.write(self.encode().as_bytes());

                let mut response = Response::new(Protocol::HTTP0_9);

                loop {
                    let mut resp: [u8; 512] = [0; 512];
                    let bytes_read = stream.read(&mut resp).unwrap();

                    if bytes_read == 0 {
                        break;
                    }

                    println!("{}", str::from_utf8(&resp).unwrap());
                }

                Ok(response)
            }
            _ => todo!(),
        }
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Messages#http_responses
/// You will notice that most fields are Option'd even though it may seem like they shouldn't be
/// This is because in HTTP/0.9 the response consists only of the body, so other fields must be set
/// to None
#[derive(Default)]
pub struct Response {
    pub protocol: Option<Protocol>,
    pub status_code: Option<u32>,
    pub reason: Option<String>,

    pub headers: Vec<Header>,

    /// Most responses have bodies, but certain responses (201 Created, 204 No Content) don't
    pub body: Option<String>,

    /// The protocol the response is expected to be of, taken from the request protocol
    /// This is allowed to be an Option solely so that Default works, it's value can be confidently
    /// unwrapped without second thought about handling None
    expected_protocol: Option<Protocol>,
}

impl Response {
    fn new(expected_protocol: Protocol) -> Self {
        Self {
            expected_protocol: Some(expected_protocol),
            ..Default::default()
        }
    }

    fn decode_body_chunk(&mut self, chunk: &[u8]) {
        let mut body = self.body.clone().unwrap_or(String::new());
        body.push_str(str::from_utf8(chunk).unwrap());

        if body.ends_with("\r\n\r\n") {
            body = body.strip_suffix("\r\n\r\n").unwrap().to_string();
        }

        self.body = Some(body);
    }
}

#[derive(Default, Clone)]
pub struct Client {
    addr: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn connect_to(&mut self, addr: String) -> Self {
        self.addr = Some(addr);
        self.clone()
    }

    pub fn send_request(self: &Self, request: Request) -> Option<Response> {
        match request.send(self.addr.clone().unwrap()) {
            Ok(resp) => Some(resp),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }
}
