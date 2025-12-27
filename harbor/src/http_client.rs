use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

pub const MINIMUM_CHUNK_LENGTH: usize = 8;
pub const CHUNK_LENGTH: usize = 512;

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
#[derive(Debug)]
pub enum Protocol {
    HTTP0_9,
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

impl Protocol {
    pub fn connect(&self, addr: String) -> Option<TcpStream> {
        match self {
            Protocol::HTTP0_9 | Protocol::HTTP1_0 | Protocol::HTTP1_1 => {
                TcpStream::connect(addr).ok()
            }
            _ => None,
        }
    }
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

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,

    is_complete: bool,
}

impl Header {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            is_complete: false,
        }
    }

    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            is_complete: true,
        }
    }

    pub fn mark_complete(&mut self) {
        self.is_complete = true;
    }
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
        match self.protocol {
            Protocol::HTTP0_9 => {
                format!("{} {}\r\n\r\n", self.method, self.request_target)
            }
            _ => {
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
    }
}

impl Request {
    fn ensure_integrity(&self, client: &Client) -> Result<(), RequestIntegrityError> {
        match self.protocol {
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Evolution_of_HTTP#http0.9_%E2%80%93_the_one-line_protocol
            Protocol::HTTP0_9 => {
                if self.method != "GET" && !client.permissive {
                    return Err(RequestIntegrityError {
                        kind: RequestIntegrityErrorKind::InvalidMethod,
                        message: format!(
                            "Only allowed method in HTTP/0.9 is 'GET', not {}",
                            self.method
                        ),
                    });
                }

                if self.headers.len() != 0 && !client.permissive {
                    return Err(RequestIntegrityError {
                        kind: RequestIntegrityErrorKind::InvalidHeaders,
                        message: format!(
                            "HTTP/0.9 must take 0 header, found {}",
                            self.headers.len()
                        ),
                    });
                }

                // Strict condition that body must not be present
                // Independent of permissibility setting of client
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

    fn send(&self, client: &Client) -> Result<Response, RequestIntegrityError> {
        let integrity = self.ensure_integrity(client);
        if integrity.is_err() {
            return Err(integrity.unwrap_err());
        }

        match self.protocol {
            Protocol::HTTP0_9 => {
                let mut stream = client.connection.as_ref().unwrap();
                _ = stream.write(self.encode().as_bytes());

                let mut response = Response::new();

                loop {
                    let mut chunk = [0; CHUNK_LENGTH];
                    let bytes_read = stream.read(&mut chunk).unwrap();

                    if bytes_read == 0 {
                        break;
                    }

                    response.decode_body_chunk(&chunk);
                }

                response.strip_zeros();

                Ok(response)
            }
            Protocol::HTTP1_0 | Protocol::HTTP1_1 => {
                let mut stream = client.connection.as_ref().unwrap();
                _ = stream.write(self.encode().as_bytes());

                let mut response_decoder = ResponseDecoder::new();

                loop {
                    let mut resp: [u8; 512] = [0; 512];
                    let bytes_read = stream.read(&mut resp).unwrap();

                    if bytes_read == 0 {
                        break;
                    }

                    response_decoder.decode(&resp);

                    println!("{}", str::from_utf8(&resp).unwrap());
                }

                // println!("{}")

                Ok(response_decoder.response)
            }
            _ => todo!(),
        }
    }
}

pub enum ResponseDecoderState {
    /// Atomic type - response data reads must be in chunks that are big enough to read the entire
    /// protocol in a single line (i.e. 8 characters minimum)
    Protocol,

    /// Atomic type
    Status,

    /// May be read through multiple read operations
    Reason,

    /// May be read through multiple read operations
    HeaderName,

    /// May be read through multiple read operations
    HeaderValue,

    /// Expected to be read through multiple read operations
    Body,
}

pub struct ResponseDecoder {
    state: ResponseDecoderState,
    response: Response,
}

impl ResponseDecoder {
    pub fn new() -> Self {
        Self {
            state: ResponseDecoderState::Protocol,
            response: Response::default(),
        }
    }

    pub fn decode(&mut self, data: &[u8]) {
        let string_data = String::from_utf8(data.to_vec())
            .unwrap()
            .trim_start_matches(' ')
            .to_string();

        match self.state {
            ResponseDecoderState::Protocol => {
                assert!(string_data.len() > MINIMUM_CHUNK_LENGTH);

                let (protocol, remaining) = string_data.split_once(" ").unwrap();
                self.response.protocol = Some(match protocol {
                    "HTTP/1.0" => Protocol::HTTP1_0,
                    "HTTP/1.1" => Protocol::HTTP1_1,
                    "HTTP/2.0" => Protocol::HTTP2_0,
                    "HTTP/3.0" => Protocol::HTTP3_0,
                    _ => panic!("Unexpected protocol received: {}", protocol),
                });

                self.state = ResponseDecoderState::Status;

                self.decode(remaining.as_bytes());
            }
            // All following states may be false alerts, brought about by a recursion when the
            // string data is insufficient to properly construct the required field
            // Consequently, their handlers start with a validity check
            ResponseDecoderState::Status => {
                if string_data.len() < 3 {
                    return;
                }

                let (status, remaining) = string_data.split_once(" ").unwrap();
                self.response.status_code = Some(status.to_string().parse::<u32>().unwrap());

                self.state = ResponseDecoderState::Reason;

                self.decode(remaining.as_bytes());
            }

            ResponseDecoderState::Reason => {
                if string_data.len() == 0 {
                    return;
                }

                if string_data.starts_with("\r\n") {
                    // Reason complete
                    self.state = ResponseDecoderState::HeaderName;

                    let remaining = string_data.strip_prefix("\r\n").unwrap();

                    return self.decode(remaining.as_bytes());
                }

                if string_data.contains("\r\n") {
                    // Data contains atleast last part of reason
                    let (reason, remaining) = string_data.split_once("\r\n").unwrap();

                    match &self.response.reason {
                        // Data contains last part of reason
                        Some(curr_reason) => {
                            self.response.reason = Some(curr_reason.to_owned() + reason)
                        }
                        // Data contains entire reason
                        None => {
                            self.response.reason = Some(reason.to_string());
                        }
                    }

                    self.state = ResponseDecoderState::HeaderName;

                    return self.decode(remaining.as_bytes());
                } else {
                    // Data contains only partial reason
                    match &self.response.reason {
                        // Middle part of reason
                        Some(curr_reason) => {
                            self.response.reason =
                                Some(curr_reason.to_owned() + string_data.as_str());
                        }
                        // First part of reason
                        None => {
                            self.response.reason = Some(string_data);
                        }
                    }
                }
            }

            ResponseDecoderState::HeaderName => {
                if string_data.len() == 0 {
                    return;
                }

                if string_data.starts_with(":") {
                    // End of header name
                    let remaining = string_data.strip_prefix(":").unwrap().trim_start();

                    self.state = ResponseDecoderState::HeaderValue;

                    return self.decode(remaining.as_bytes());
                }

                if string_data.starts_with("\r\n") {
                    // End of headers
                    let remaining = string_data.strip_prefix("\r\n").unwrap();

                    self.state = ResponseDecoderState::Body;

                    return self.decode(remaining.as_bytes());
                }

                if string_data.contains(":") {
                    // Last part of name in string data
                    let (name_data, remaining) = string_data.split_once(":").unwrap();

                    match self.response.headers.last_mut() {
                        Some(previous) => {
                            if !previous.is_complete {
                                previous.name += name_data;
                            } else {
                                let mut new_header = Header::empty();
                                new_header.name += name_data;

                                self.response.headers.push(new_header);
                            }
                        }
                        None => {
                            let mut new_header = Header::empty();
                            new_header.name += name_data;

                            self.response.headers.push(new_header);
                        }
                    }

                    self.state = ResponseDecoderState::HeaderValue;

                    return self.decode(remaining.as_bytes());
                } else {
                    // First or middle part of name in string data
                    match self.response.headers.last_mut() {
                        Some(previous) => {
                            if !previous.is_complete {
                                // Middle part
                                previous.name += string_data.as_str();
                            } else {
                                // First part
                                let mut new_header = Header::empty();
                                new_header.name = string_data;

                                self.response.headers.push(new_header);
                            }
                        }
                        None => {
                            // First part
                            let mut new_header = Header::empty();
                            new_header.name = string_data;

                            self.response.headers.push(new_header);
                        }
                    }
                }
            }

            ResponseDecoderState::HeaderValue => {
                if string_data.len() == 0 {
                    return;
                }

                if string_data.starts_with("\r\n") {
                    // header complete
                    let remaining = string_data.strip_prefix("\r\n").unwrap();

                    let last = self.response.headers.last_mut().unwrap();
                    last.mark_complete();

                    self.state = ResponseDecoderState::HeaderName;

                    return self.decode(remaining.as_bytes());
                }

                if string_data.contains("\r\n") {
                    // Last part of name in string data
                    let (value_data, remaining) = string_data.split_once("\r\n").unwrap();

                    let previous = self.response.headers.last_mut().unwrap();
                    previous.value += value_data;
                    previous.mark_complete();

                    self.state = ResponseDecoderState::HeaderName;

                    return self.decode(remaining.as_bytes());
                } else {
                    // First or middle part of value in string data
                    let previous = self.response.headers.last_mut().unwrap();
                    previous.value += string_data.as_str();
                }
            }

            ResponseDecoderState::Body => {
                if string_data.len() == 0 {
                    return;
                }

                match &self.response.body {
                    Some(body) => self.response.body = Some(body.to_owned() + string_data.as_str()),
                    None => self.response.body = Some(string_data),
                }
            }
        }
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Messages#http_responses
/// You will notice that most fields are Option'd even though it may seem like they shouldn't be
/// This is because in HTTP/0.9 the response consists only of the body, so other fields must be set
/// to None
#[derive(Default, Debug)]
pub struct Response {
    pub protocol: Option<Protocol>,
    pub status_code: Option<u32>,
    pub reason: Option<String>,

    pub headers: Vec<Header>,

    /// Most responses have bodies, but certain responses (201 Created, 204 No Content) don't
    pub body: Option<String>,
}

impl Response {
    fn new() -> Self {
        Default::default()
    }

    fn decode_body_chunk(&mut self, chunk: &[u8]) {
        let mut body = self.body.clone().unwrap_or(String::new());
        body.push_str(str::from_utf8(chunk).unwrap());

        if body.ends_with("\r\n\r\n") {
            body = body.strip_suffix("\r\n\r\n").unwrap().to_string();
        }

        self.body = Some(body);
    }

    fn strip_zeros(&mut self) {
        match &self.body {
            Some(body) => {
                self.body = Some(body.trim_end_matches("\0").to_string());
            }
            None => return,
        }
    }
}

#[derive(Default)]
pub struct Client {
    addr: Option<String>,

    connection: Option<TcpStream>,
    preferred_protocol: Option<Protocol>,

    permissive: bool,
}

impl Client {
    pub fn new(prefers: Protocol, permissive: bool) -> Self {
        Self {
            preferred_protocol: Some(prefers),
            permissive,
            ..Default::default()
        }
    }

    pub fn connect_to(&mut self, addr: String) {
        self.addr = Some(addr.clone());
        match &self.preferred_protocol {
            Some(proto) => {
                self.connection = proto.connect(addr);
            }
            None => {
                self.preferred_protocol = Some(Protocol::HTTP1_1);
                self.connection = self.preferred_protocol.as_ref().unwrap().connect(addr);
            }
        }
    }

    pub fn send_request(&self, request: Request) -> Option<Response> {
        match request.send(self) {
            Ok(resp) => Some(resp),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }
}
