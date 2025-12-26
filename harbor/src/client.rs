use std::io::{Read, Write};
use std::net::TcpStream;

trait ReqEncodable {
    fn encode(&self) -> String;
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
            Self::HTTP0_9 => "HTTP/0.9",
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

pub struct Response {}

pub fn send_request(request: Request) -> Response {
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();

    _ = stream.write(request.encode().as_bytes());

    loop {
        let mut resp: [u8; 512] = [0; 512];
        let bytes_read = stream.read(&mut resp).unwrap();

        if bytes_read == 0 {
            break;
        }

        println!("{}", str::from_utf8(&resp).unwrap());
    }

    Response {}
}
