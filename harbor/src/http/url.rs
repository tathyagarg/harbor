use std::num::{IntErrorKind, ParseIntError};

use encoding_rs;
use idna;

use crate::infra::Serializable;

pub const FORBIDDEN_HOST_CODE_POINTS: [char; 17] = [
    '\u{0000}', '\u{0009}', '\u{000a}', '\u{000d}', '\u{0020}', '\u{0023}', '\u{002f}', '\u{003a}',
    '\u{003c}', '\u{003e}', '\u{003f}', '\u{0040}', '\u{005b}', '\u{005c}', '\u{005d}', '\u{005e}',
    '\u{007c}',
];

pub fn is_url_codepoint(c: char) -> bool {
    matches!(c,
        'a'..='z' |
        'A'..='Z' |
        '0'..='9' |
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' |
        '.' | '/' | ':' | ';' | '=' | '?' | '@' | '_' | '~' |
        '\u{A0}'..='\u{D7FF}' | '\u{E000}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' |
        '\u{10000}'..='\u{1FFFD}' | '\u{20000}'..='\u{2FFFD}' |
        '\u{30000}'..='\u{3FFFD}' | '\u{40000}'..='\u{4FFFD}' |
        '\u{50000}'..='\u{5FFFD}' | '\u{60000}'..='\u{6FFFD}' |
        '\u{70000}'..='\u{7FFFD}' | '\u{80000}'..='\u{8FFFD}' |
        '\u{90000}'..='\u{9FFFD}' | '\u{A0000}'..='\u{AFFFD}' |
        '\u{B0000}'..='\u{BFFFD}' | '\u{C0000}'..='\u{CFFFD}' |
        '\u{D0000}'..='\u{DFFFD}' | '\u{E1000}'..='\u{EFFFD}' |
        '\u{F0000}'..='\u{FFFFD}' | '\u{100000}'..='\u{10FFFD}')
}

pub const SPECIAL_SCHEMES: [&'static str; 6] = ["ftp", "file", "http", "https", "ws", "wss"];

fn is_special_scheme(scheme: &String) -> bool {
    SPECIAL_SCHEMES.contains(&scheme.as_str())
}

pub fn special_scheme_default_port(scheme: &String) -> Option<u16> {
    if !is_special_scheme(scheme) {
        return None;
    };

    return match scheme.as_str() {
        "ftp" => Some(21),
        "file" => None,
        "http" => Some(80),
        "https" => Some(443),
        "ws" => Some(80),
        "wss" => Some(443),
        _ => unreachable!(),
    };
}

fn _is_windows_drive_letter(codepoint: &String, second: &[char]) -> bool {
    let mut iter = codepoint.chars();

    iter.next().unwrap().is_ascii_alphabetic() && second.contains(&iter.next().unwrap())
}

fn is_windows_drive_letter(codepoint: &String) -> bool {
    _is_windows_drive_letter(codepoint, &[':', '|'])
}

fn is_normalized_windows_drive_letter(codepoint: &String) -> bool {
    _is_windows_drive_letter(codepoint, &[':'])
}

fn starts_with_windows_drive_letter(string: &String) -> bool {
    string.chars().count() >= 2
        && is_windows_drive_letter(string)
        && (string.chars().count() == 2
            || matches!(string.chars().nth(2).unwrap(), '/' | '\\' | '?' | '#'))
}

fn is_single_dot(segment: &String) -> bool {
    matches!(segment.to_ascii_lowercase().as_str(), "." | "%2e")
}

fn is_double_dot(segment: &String) -> bool {
    matches!(
        segment.to_ascii_lowercase().as_str(),
        ".." | ".%2e" | "%2e." | "%2e%2e"
    )
}

struct StringPointer {
    chars: Vec<char>,
    pointer: isize,
    c: char,
    remaining: Vec<char>,

    is_eof: bool,
}

impl StringPointer {
    pub fn new(target: String) -> StringPointer {
        let chars = target.chars().collect::<Vec<char>>();

        let pointer = 0;
        let c = 0 as char;
        let remaining = vec![];

        let mut p = StringPointer {
            chars,
            pointer,
            c,
            remaining,
            is_eof: false,
        };
        p.update_c();

        p
    }

    pub fn update_c(&mut self) {
        if self.pointer + 1 > (self.chars.len() as isize) {
            self.is_eof = true;
            self.c = '\0';
            self.remaining = vec![];
        } else {
            self.c = self.chars[self.pointer as usize];
            self.remaining = self.chars[self.pointer as usize + 1..].to_vec();
        }
    }

    pub fn advance_by(&mut self, size: isize) {
        self.pointer += size;
        self.update_c();
    }
}

fn char_as_hex(c: char) -> u8 {
    if c >= '0' && c <= '9' {
        return (c as u8) - b'0';
    } else if c >= 'a' && c <= 'f' {
        return (c as u8) - b'a';
    } else if c >= 'A' && c <= 'F' {
        return (c as u8) - b'A';
    }

    return 0;
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IPv4(u32);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IPv6([u16; 8]);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum IPAddress {
    IPv4(IPv4),
    IPv6(IPv6),
}

#[derive(Debug)]
pub enum IPv6ParseError {
    IPv6InvalidCompression,
    IPv6TooManyPieces,
    IPv6MultipleCompression,
    IPv4InIPv6InvalidCodePoint,
    IPv4InIPv6TooManyPieces,
    IPv4InIPv6OutOfRange,
    IPv4InIPv4TooFewParts,
    IPv6InvalidCodePoint,
    IPv6TooFewPieces,
}

impl IPv6 {
    /// https://url.spec.whatwg.org/#concept-ipv6-parser
    pub fn parse(input: String) -> Result<IPv6, IPv6ParseError> {
        let mut address = IPv6([0; 8]);
        let mut piece_index: usize = 0;
        let mut compress: Option<usize> = None;

        let mut pointer = StringPointer::new(input.clone());

        if pointer.c == ':' {
            if pointer.remaining[0] != ':' {
                return Err(IPv6ParseError::IPv6InvalidCompression);
            }

            pointer.advance_by(2);
            piece_index += 1;
            compress = Some(piece_index);
        }

        while !pointer.is_eof {
            if piece_index == 8 {
                return Err(IPv6ParseError::IPv6TooManyPieces);
            }

            if pointer.c == ':' {
                if compress.is_some() {
                    return Err(IPv6ParseError::IPv6MultipleCompression);
                }

                pointer.advance_by(1);
                piece_index += 1;
                compress = Some(piece_index);
            }

            let mut value: u16 = 0;
            let mut length = 0;

            while length < 4 && pointer.c.is_ascii_hexdigit() {
                value = value * 0x10 + char_as_hex(pointer.c) as u16;
                pointer.advance_by(1);
                length += 1;
            }

            if pointer.c == '.' {
                if length == 0 {
                    return Err(IPv6ParseError::IPv4InIPv6InvalidCodePoint);
                }

                pointer.advance_by(-length);
                if piece_index > 6 {
                    return Err(IPv6ParseError::IPv4InIPv6TooManyPieces);
                }

                let mut numbers_seen = 0;

                while !pointer.is_eof {
                    let mut ipv4_piece: Option<u8> = None;

                    if numbers_seen > 0 {
                        if pointer.c == '.' && numbers_seen < 4 {
                            pointer.advance_by(1);
                        } else {
                            return Err(IPv6ParseError::IPv4InIPv6InvalidCodePoint);
                        }
                    }

                    if !pointer.c.is_ascii_digit() {
                        return Err(IPv6ParseError::IPv4InIPv6InvalidCodePoint);
                    }

                    while pointer.c.is_ascii_digit() {
                        let number = pointer.c as u8 - b'0';
                        if ipv4_piece.is_none() {
                            ipv4_piece = Some(number);
                        } else {
                            let curr = ipv4_piece.unwrap();

                            if curr == 0 {
                                return Err(IPv6ParseError::IPv4InIPv6InvalidCodePoint);
                            } else {
                                ipv4_piece = curr.checked_mul(10);
                                if ipv4_piece.is_some() {
                                    ipv4_piece = ipv4_piece.unwrap().checked_add(number);
                                }
                            }
                        }

                        if ipv4_piece.is_none() {
                            return Err(IPv6ParseError::IPv4InIPv6OutOfRange);
                        }
                        pointer.advance_by(1);
                    }

                    address.0[piece_index] =
                        address.0[piece_index] * 0x100 + ipv4_piece.unwrap() as u16;

                    numbers_seen += 1;
                    if numbers_seen == 2 || numbers_seen == 4 {
                        piece_index += 1;
                    }
                }

                if numbers_seen != 4 {
                    return Err(IPv6ParseError::IPv4InIPv4TooFewParts);
                }

                break;
            } else if pointer.c == ':' {
                pointer.advance_by(1);
                if pointer.is_eof {
                    return Err(IPv6ParseError::IPv6InvalidCodePoint);
                }
            } else if !pointer.is_eof {
                return Err(IPv6ParseError::IPv6InvalidCodePoint);
            }

            address.0[piece_index] = value;
            piece_index += 1;
        }

        if let Some(comp) = compress {
            let mut swaps = piece_index - comp;
            piece_index = 7;
            while piece_index != 0 && swaps > 0 {
                let temp = address.0[piece_index];
                address.0[piece_index] = address.0[comp + swaps - 1];
                address.0[comp + swaps - 1] = temp;

                piece_index -= 1;
                swaps -= 1;
            }
        } else if piece_index != 8 {
            return Err(IPv6ParseError::IPv6TooFewPieces);
        }

        Ok(address)
    }

    fn compressed_piece_index(&self) -> usize {
        let mut longest_index = None;
        let mut longest_size = 1;
        let mut found_index: Option<usize> = None;
        let mut found_size = 0;

        for piece_index in 0..8 {
            if self.0[piece_index] != 0 {
                if found_size > longest_size {
                    longest_index = found_index;
                    longest_size = found_size;
                }

                found_index = None;
                found_size = 0;
            } else {
                if found_index.is_none() {
                    found_index = Some(piece_index);
                    found_size += 1;
                }
            }
        }

        if found_size > longest_size {
            return found_index.unwrap();
        }

        longest_index.unwrap()
    }
}

impl Serializable for IPv6 {
    fn serialize(&self) -> String {
        let mut output = String::new();
        let compress = self.compressed_piece_index();
        let mut ignore0 = false;

        for piece_index in 0..8 {
            if ignore0 && self.0[piece_index] == 0 {
                continue;
            } else if ignore0 {
                ignore0 = false;
            }

            if compress == piece_index {
                let separator = if piece_index == 0 { "::" } else { ":" };
                output.push_str(separator);

                ignore0 = true;
                continue;
            }

            output.push_str(format!("{:x}", self.0[piece_index]).as_str());
            if piece_index != 7 {
                output.push(':');
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_valid() {
        let addresses = [
            "::",
            "::1",
            "::ffff:192.0.2.128",
            "0:0:0:0:0:0:0:1",
            "1050:0:0:0:5:600:300c:1",
            "2001::1",
            "2001:0:9D38:953C:10EF:EE22:FFDD:AABB",
            "2001:0DA8:0200:0012:0000:00B8:0000:02AA",
            "2001:0db8::1",
            "2001:0db8::1:0:0:1",
            "2001:0DB8::4152:EBAF:CE01:0001",
            "2001:0db8:0:0:1:0:0:1",
            "2001:0DB8:0000:CD30:0000:0000:0000:0000",
            "2001:0DB8:1234:5678:ABCD:EF01:2345:6789",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "2001:0db8:85a3:08d3:1319:8a2e:0370:7344",
            "2001:0DB8:aaaa:0007:0000:0000:0000:0001",
            "2001:2::10",
            "2001:44b8:4126:f600:91bd:970c:9073:12df",
            "2001:4860:4860::8888",
            "2001:500:2d::d",
            "2001:558:fc03:11:5e63:3eff:fe67:edf9",
            "2001:acad:abad:1::bc",
            "2001:b50:ffd3:76:ce58:32ff:fe00:e7",
            "2001:db8::0:1:0:0:1",
            "2001:db8::1",
            "2001:db8::1:0:0:1",
            "2001:db8::212:7403:ace0:1",
            "2001:DB8::4:5:6:7",
            "2001:db8::5",
            "2001:DB8::8:800:200C:417A",
            "2001:db8::aaaa:0:0:1",
            "2001:db8:0::1",
            "2001:db8:0:0::1",
            "2001:db8:0:0:0::1",
            "2001:db8:0:0:1::1",
            "2001:DB8:0:0:1::1",
            "2001:db8:0:0:1:0:0:1",
            "2001:DB8:0:0:8:800:200C:417A",
            "2001:db8:0:0:aaaa::1",
            "2001:db8:0000:0:1::1",
            "2001:db8:3c4d:15::1",
            "2001:DB8:85A3::8A2E:370:7334",
            "2001:db8:aaaa:bbbb:cccc:dddd::1",
            "2001:db8:aaaa:bbbb:cccc:dddd:0:1",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:0001",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:001",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:01",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:1",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:aaaa",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:AAAA",
            "2001:db8:aaaa:bbbb:cccc:dddd:eeee:AaAa",
            "2001:db8:d03:bd70:fede:5c4d:8969:12c4",
            "2002::8364:7777",
            "2002:4559:1FE2::4559:1FE2",
            "2002:C000:203:200::",
            "2002:cb0a:3cdd:1:1:1:1:1",
            "2400:8902::f03c:92ff:feb5:f66d",
            "2400:c980:0:e206:b07d:8cf9:2b05:fb06",
            "2400:cb00:2048:1::6814:507",
            "2404:6800:4009:805::2004",
            "2607:f8b0:4005:80b::200e",
            "2607:f8b0:400a:809::200e",
            "2620:0:1cfe:face:b00c::3",
            "2620:0:2d0:200::7",
            "3fff:ffff:3:1:0:0:0:7",
            "ABCD:EF01:2345:6789:ABCD:EF01:2345:6789",
            "fc00::",
            "fd3b:d101:e37f:9713::1",
            "fd44:a77b:40ca:db17:37df:f4c4:f38a:fc81",
            "fe80::",
            "FE80:0000:0000:0000:0202:B3FF:FE1E:8329",
            "fec0:0:0:1::1",
            "FEDC:BA98:7654:3210:FEDC:BA98:7654:3210",
            "FF01::101",
            "FF01:0:0:0:0:0:0:1",
            "FF01:0:0:0:0:0:0:101",
            "FF02::1",
            "FF02:0:0:0:0:0:0:1",
            "FF02:0:0:0:0:0:0:a",
            "FF05:15:25:df:20:4a:b4:24",
            "FF08:0:0:0:0:0:0:fc",
        ];

        for addr in addresses {
            assert!(IPv6::parse(addr.to_string()).is_ok());
        }
    }

    #[test]
    fn test_invalid() {
        let addresses = [
            "::-1",
            "::/0/0",
            "::%eth0",
            "::ffff:0.0.0.256",
            "::ffff:127.0.0.1/96",
            "::ffff:192.0.2.128/33",
            "::ffff:192.0.2.256",
            "::ffff:192.168.1.256",
            "1:2:3:4:5:6:7:8:9",
            "1080:0:0:0:0:0:0:192.88.99",
            "2001::0223:dead:beef::1",
            "2001::dead::beef",
            "2001::ff4:2:1:1:1:1:1",
            "2001:0DB8:0:CD3",
            "2001:0db8:1234:5678:90AB:CDEF:0012:3456:789a",
            "2001:db8::1 ::2",
            "2001:db8:/60",
            "2001:db8:0:0:0:0:0/64",
            "2001:db8:0:0:0:0:f:1g",
            "2001:db8:0:0:0g00:1428:57ab",
            "2001:db8:0:1::/129",
            "2001:db8:0:1::1::1",
            "2001:db8:0:1::a:b:c:d:e:f",
            "2001:db8:0:1:/64",
            "2001:db8:0:1:1:1:1:1:1",
            "2001:db8:0:1:1:1:1:1#test",
            "2001:db8:0:1g:0:0:0:1",
            "2001:db8:aaaa:bbbb:cccc:dddd-eeee:ffff",
            "2001:db8:aaaa:bbbb:cccc:dddd-eeee:ffff",
            "2001:dg8:0:0:0:0:1428:57ab",
            "2001:dg8:0:0:0:0:1428:57ab",
            "2001:gdba:0000:0000:0000:0000:3257:9652",
            "2001:gdba:0000:0000:0000:0000:3257:9652",
            "2001:ggg:0:0:0:0:1428:57ab",
            "2001:ggg:0:0:0:0:1428:57ab",
            "2001.x:0:0:0:0:0:0:1",
            "20011:db8:0:1:1:1:1:1",
            "2403:780:f:102:a:a:1:0:0",
            "2403:780:f:102:a:a:1:0:0",
            "2403:780:f:102:g:a:1:0",
            "2403:780:f:102:g:a:1:0",
            "260.02:00a:b:10:abc:def:123f:2552",
            "260.02:00a:b:10:abc:def:123f:2552",
            "fe80::/130",
            "fe80::/130",
            "fe80::7::8",
            "fe80::7::8",
            "2001:0DB8:0:CD3",
        ];

        for addr in addresses {
            assert!(IPv6::parse(addr.to_string()).is_err());
        }
    }
}

#[derive(Debug)]
pub enum IPv4ParseError {
    NumberParsingError,
    IPv4TooManyParts,
    IPv4NonNumericPart,
    IPv4OutOfRange,
}

impl IPv4 {
    fn parse_number(num: &str) -> Result<(u32, bool), IPv4ParseError> {
        if num.trim().is_empty() {
            return Err(IPv4ParseError::NumberParsingError);
        }

        let mut input = String::from(num);

        let mut validation_error = false;
        let mut r = 10;

        if num.chars().count() >= 2 && (&num[0..2] == "0x" || &num[0..2] == "0X") {
            validation_error = true;
            input = num.chars().into_iter().skip(2).collect();
            r = 16;
        }

        if num.chars().count() >= 2 && &num[0..1] == "0" {
            validation_error = true;
            input = num.chars().into_iter().skip(1).collect();
            r = 8;
        }

        if input.is_empty() {
            return Ok((0, true));
        }

        match u32::from_str_radix(&input, r) {
            Err(_) => Err(IPv4ParseError::NumberParsingError),
            Ok(output) => Ok((output, validation_error)),
        }
    }

    pub fn parse(input: String) -> Result<IPv4, IPv4ParseError> {
        let mut parts = input.split('.').collect::<Vec<&str>>();
        if parts.clone().last().unwrap().trim().is_empty() {
            if parts.len() > 1 {
                _ = parts.pop();
            }
        }

        if parts.len() > 4 {
            return Err(IPv4ParseError::IPv4TooManyParts);
        }

        let mut numbers = Vec::<u32>::new();

        for part in parts {
            let result = IPv4::parse_number(part);
            if result.is_err() {
                return Err(IPv4ParseError::IPv4NonNumericPart);
            }

            let (num, _) = result.unwrap();
            numbers.push(num);
        }

        for (i, num) in numbers.iter().enumerate() {
            if *num > 255 && i != numbers.len() - 1 {
                return Err(IPv4ParseError::IPv4OutOfRange);
            }

            if i == numbers.len() - 1 && *num > 256u32.pow((5 - numbers.len()) as u32) {
                return Err(IPv4ParseError::IPv4OutOfRange);
            }
        }

        let mut ipv4 = numbers.last().unwrap().to_owned();
        _ = numbers.pop();

        for (i, n) in numbers.iter().enumerate() {
            ipv4 += *n * 256u32.pow((3 - i) as u32);
        }

        Ok(IPv4(ipv4))
    }
}

impl Serializable for IPv4 {
    fn serialize(&self) -> String {
        let mut output = String::new();
        let mut n = self.0;

        for i in 1..=4 {
            output.insert_str(0, (n % 256).to_string().as_str());
            if i != 4 {
                output.insert(0, '.');
            }

            n = n / 256
        }

        output
    }
}

type PercentEncodeSet = dyn Fn(u8) -> bool;

fn c0_percent_encode_set(b: u8) -> bool {
    b <= 0x1f || b > 0x7e
}

fn query_percent_encode_set(b: u8) -> bool {
    c0_percent_encode_set(b) || matches!(b, 0x20 | 0x22 | 0x23 | 0x3c | 0x3e)
}

fn special_query_percent_encode_set(b: u8) -> bool {
    query_percent_encode_set(b) || b == 0x27
}

fn path_percent_encode_set(b: u8) -> bool {
    query_percent_encode_set(b) || matches!(b, 0x3f | 0x5e | 0x60 | 0x7b | 0x7d)
}

fn fragment_percent_encode_set(b: u8) -> bool {
    c0_percent_encode_set(b) || matches!(b, 0x20 | 0x22 | 0x3c | 0x3e | 0x60)
}

pub fn percent_encoding_after_encoding(
    encoding: &'static encoding_rs::Encoding,
    input: &String,
    percent_encode_set: &PercentEncodeSet,
    space_as_plus: Option<bool>,
) -> String {
    let mut output = String::new();
    let (bytes, _, _) = encoding.encode(&input);

    for b in bytes.into_iter() {
        if space_as_plus.unwrap_or(false) && *b == b' ' {
            output.push('+');
        } else {
            let isomporh = *b as char;

            if !percent_encode_set(*b) {
                output.push(isomporh);
            } else {
                output.push_str(&format!("%{:02X}", b));
            }
        }
    }

    output
}

pub fn percent_decode(bytes: &[u8]) -> Vec<u8> {
    let mut output = vec![];
    let mut skip = 0;

    for (i, byte) in bytes.iter().enumerate() {
        if skip != 0 {
            skip -= 1;
            continue;
        }

        if *byte != b'%' {
            output.push(*byte);
        } else if !bytes[i + 1].is_ascii_hexdigit() || !bytes[i + 2].is_ascii_hexdigit() {
            output.push(*byte);
        } else {
            let hi = bytes[i + 1];
            let lo = bytes[i + 1];

            let hex = |b| match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => unreachable!(),
            };

            let byte_point = (hex(hi) << 4) | hex(lo);
            output.push(byte_point);
            skip = 2;
        }
    }

    output
}

pub fn utf8_decode_no_bom(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap()
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Opaque(String);

#[derive(Debug)]
pub enum OpaqueParseError {
    HostInvalidCodePoint,
    InvalidURLUnit,
}

impl Opaque {
    pub fn parse(input: &String) -> Result<Opaque, OpaqueParseError> {
        for forbidden in FORBIDDEN_HOST_CODE_POINTS {
            if input.contains(forbidden) {
                return Err(OpaqueParseError::HostInvalidCodePoint);
            }
        }

        let chars = input.chars().into_iter().collect::<Vec<char>>();

        for (i, c) in input.chars().enumerate() {
            if !is_url_codepoint(c) && c != '%' {
                return Err(OpaqueParseError::InvalidURLUnit);
            }

            if c == '%' && !(chars[i - 1].is_ascii_hexdigit() && chars[i + 1].is_ascii_hexdigit()) {
                return Err(OpaqueParseError::InvalidURLUnit);
            }
        }

        let result = percent_encoding_after_encoding(
            encoding_rs::Encoding::for_label(b"utf-8").unwrap(),
            input,
            &c0_percent_encode_set,
            None,
        );

        Ok(Opaque(result))
    }
}

type URLPathSegment = String;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum URLPath {
    Segment(URLPathSegment),
    List(Vec<URLPathSegment>),
}

impl URLPath {
    fn is_opaque(&self) -> bool {
        matches!(self, URLPath::Segment(_))
    }

    fn _ensure_list(&self) -> &Vec<URLPathSegment> {
        assert!(!self.is_opaque());

        match self {
            URLPath::Segment(_) => unreachable!(),
            URLPath::List(l) => l,
        }
    }

    fn _ensure_list_mut(&mut self) -> &mut Vec<URLPathSegment> {
        assert!(!self.is_opaque());

        match self {
            URLPath::Segment(_) => unreachable!(),
            URLPath::List(l) => l,
        }
    }

    fn push(&mut self, elem: URLPathSegment) {
        self._ensure_list_mut().push(elem);
    }

    fn is_empty_list(&self) -> bool {
        self._ensure_list().is_empty()
    }
}

impl Default for URLPath {
    fn default() -> Self {
        Self::List(Vec::<URLPathSegment>::new())
    }
}

impl Serializable for URLPath {
    fn serialize(&self) -> String {
        if let URLPath::Segment(segment) = self {
            return segment.clone();
        }

        let mut output = String::new();
        for segment in self._ensure_list() {
            output.push_str(format!("/{}", segment).as_str());
        }

        output
    }
}

pub type Domain = String;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Host {
    Domain(Domain),
    IPAddress(IPAddress),
    Opaque(Opaque),
    Empty,
}

#[derive(Debug)]
pub enum HostParseError {
    IPv6UnclosedValidation,
    IPv6ParsingError(IPv6ParseError),
    IPv4ParsingError(IPv4ParseError),
    OpaqueParseError(OpaqueParseError),
    DomainToAsciiError(idna::Errors),
}

impl Host {
    pub fn parse(input: &String, is_opaque: Option<bool>) -> Result<Host, HostParseError> {
        if input.starts_with('[') {
            if !input.ends_with(']') {
                return Err(HostParseError::IPv6UnclosedValidation);
            }

            return match IPv6::parse(input[1..input.len() - 2].to_string()) {
                Ok(ip) => Ok(Host::IPAddress(IPAddress::IPv6(ip))),
                Err(e) => Err(HostParseError::IPv6ParsingError(e)),
            };
        }

        if is_opaque.unwrap_or(false) {
            return match Opaque::parse(input) {
                Ok(opaque) => Ok(Host::Opaque(opaque)),
                Err(e) => Err(HostParseError::OpaqueParseError(e)),
            };
        }

        let domain = utf8_decode_no_bom(percent_decode(input.as_bytes()));

        let domain_ascii = idna::domain_to_ascii(&domain);

        return match domain_ascii {
            Err(e) => Err(HostParseError::DomainToAsciiError(e)),
            Ok(dom_ascii) => {
                if dom_ascii.chars().last().unwrap().is_ascii_digit() {
                    return match IPv4::parse(dom_ascii) {
                        Ok(ipv4) => Ok(Host::IPAddress(IPAddress::IPv4(ipv4))),
                        Err(e) => Err(HostParseError::IPv4ParsingError(e)),
                    };
                }

                Ok(Host::Domain(dom_ascii))
            }
        };
    }
}

impl Serializable for Host {
    fn serialize(&self) -> String {
        match self {
            Host::IPAddress(IPAddress::IPv4(ipv4)) => ipv4.serialize(),
            Host::IPAddress(IPAddress::IPv6(ipv6)) => format!("[{}]", ipv6.serialize()),
            Host::Domain(domain) => domain.clone(),
            Host::Opaque(Opaque(opaque)) => opaque.clone(),
            Host::Empty => String::new(),
        }
    }
}

// type URLPathSegment = String;
// type URLPath = Vec<URLPathSegment>;

#[derive(Debug)]
pub enum ParseURLError {
    Failure,
    MissingSchemeNonRelativeURL,
    HostMissing,
    HostParseError(HostParseError),
    PortOutOfRange,
    ParseIntError(ParseIntError),
    PortInvalid,
}

#[derive(Clone, Debug)]
pub enum ParseURLState {
    SchemeStart,
    Scheme,
    NoScheme,
    SpecialRelativeOrAuthority,
    PathOrAuthority,
    Relative,
    RelativeSlash,
    SpecialAuthoritySlashes,
    SpecialAuthorityIgnoreSlashes,
    Authority,
    Host,
    Hostname,
    Port,
    File,
    FileSlash,
    FileHost,
    PathStart,
    Path,
    OpaquePath,
    Query,
    Fragment,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct URL {
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: Option<Host>,
    pub port: Option<u16>,
    pub path: URLPath,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl URL {
    fn has_credentials(&self) -> bool {
        !self.username.is_empty() || !self.password.is_empty()
    }

    fn shorten_path(&mut self) {
        assert!(!self.path.is_opaque());

        if let URLPath::List(path) = &mut self.path {
            if self.scheme.as_str() == "file"
                && path.len() == 1
                && is_normalized_windows_drive_letter(&path[0])
            {
                return;
            }

            _ = path.pop();
        }
    }

    pub fn pure_parse(input: String) -> Result<URL, ParseURLError> {
        URL::parse(input, None, None)
    }

    pub fn parse(
        input: String,
        base: Option<URL>,
        encoding: Option<&'static encoding_rs::Encoding>,
    ) -> Result<URL, ParseURLError> {
        let maybe_url = URL::basic_url_parser(input, base, encoding, None, None);
        return match maybe_url {
            Err(e) => Err(e),
            Ok(o_url) => {
                let url = o_url.unwrap();
                if url.scheme.as_str() != "blob" {
                    return Ok(url);
                }

                todo!()
            }
        };
    }

    fn basic_url_parser(
        input: String,
        base: Option<URL>,
        mut encoding: Option<&'static encoding_rs::Encoding>,
        mut url: Option<&mut URL>,
        state_override: Option<ParseURLState>,
    ) -> Result<Option<URL>, ParseURLError> {
        let mut input = input;
        if url.is_none() {
            input = input.trim().to_string();
        }

        let mut local_url;
        let url: &mut URL = match url.as_deref_mut() {
            Some(existing) => existing,
            None => {
                local_url = URL::default();
                &mut local_url
            }
        };

        input = input.replace("\t", "").replace("\n", "");

        let mut state = state_override.clone().unwrap_or(ParseURLState::SchemeStart);
        let mut buffer = String::new();
        let mut at_sign_seen = false;
        let mut inside_brackets = false;
        let mut password_token_seen = false;

        let mut pointer = StringPointer::new(input);

        loop {
            match state {
                ParseURLState::SchemeStart => {
                    if pointer.c.is_ascii_alphabetic() {
                        buffer.push(pointer.c.to_ascii_lowercase());
                        state = ParseURLState::Scheme;
                    } else if state_override.is_none() {
                        state = ParseURLState::NoScheme;
                        pointer.advance_by(-1);
                    } else {
                        return Err(ParseURLError::Failure);
                    }
                }
                ParseURLState::Scheme => {
                    if pointer.c.is_alphanumeric() || matches!(pointer.c, '+' | '-' | '.') {
                        buffer.push(pointer.c.to_ascii_lowercase());
                    } else if pointer.c == ':' {
                        if state_override.is_some() {
                            if is_special_scheme(&url.scheme) && !is_special_scheme(&buffer) {
                                return Ok(None);
                            }

                            if !is_special_scheme(&url.scheme) && is_special_scheme(&buffer) {
                                return Ok(None);
                            }

                            if (url.has_credentials() || url.port.is_some())
                                && buffer.as_str() == "file"
                            {
                                return Ok(None);
                            }

                            if url.scheme.as_str() == "file"
                                && url.host.as_ref().is_some_and(|h| matches!(h, Host::Empty))
                            {
                                return Ok(None);
                            }
                        }

                        url.scheme = buffer.clone();

                        if state_override.is_some() {
                            if url.port.is_some_and(|port| {
                                special_scheme_default_port(&url.scheme)
                                    .is_some_and(|default| port == default)
                            }) {
                                url.port = None;
                            }
                        }

                        buffer = String::new();

                        if url.scheme.as_str() == "file" {
                            state = ParseURLState::File;
                        } else if is_special_scheme(&url.scheme)
                            && base.as_ref().is_some_and(|burl| burl.scheme == url.scheme)
                        {
                            assert!(is_special_scheme(&base.as_ref().unwrap().scheme));
                            state = ParseURLState::SpecialRelativeOrAuthority;
                        } else if is_special_scheme(&url.scheme) {
                            state = ParseURLState::SpecialAuthoritySlashes;
                        } else if pointer.remaining.starts_with(&['/']) {
                            state = ParseURLState::PathOrAuthority;
                            pointer.advance_by(1);
                        } else {
                            url.path = URLPath::Segment(String::new());
                            state = ParseURLState::OpaquePath;
                        }
                    } else if state_override.is_none() {
                        buffer = String::new();
                        state = ParseURLState::NoScheme;
                        pointer.advance_by(-pointer.pointer);
                    } else {
                        return Err(ParseURLError::Failure);
                    }
                }
                ParseURLState::NoScheme => {
                    if base.is_none()
                        || (base.as_ref().is_some_and(|burl| burl.path.is_opaque())
                            && pointer.c != '#')
                    {
                        return Err(ParseURLError::MissingSchemeNonRelativeURL);
                    } else if let Some(burl) = base.as_ref()
                        && pointer.c != '#'
                        && burl.path.is_opaque()
                    {
                        url.scheme = burl.scheme.clone();
                        url.path = burl.path.clone();
                        url.query = burl.query.clone();
                        url.fragment = Some(String::new());
                        state = ParseURLState::Fragment;
                    } else if base
                        .as_ref()
                        .is_some_and(|burl| burl.scheme.as_str() != "file")
                    {
                        state = ParseURLState::Relative;
                        pointer.advance_by(-1);
                    } else {
                        state = ParseURLState::File;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::SpecialRelativeOrAuthority => {
                    if pointer.c == '/' && pointer.remaining.starts_with(&['/']) {
                        state = ParseURLState::SpecialAuthorityIgnoreSlashes;
                        pointer.advance_by(1);
                    } else {
                        state = ParseURLState::Relative;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::PathOrAuthority => {
                    if pointer.c == '/' {
                        state = ParseURLState::Authority;
                    } else {
                        state = ParseURLState::Path;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::Relative => {
                    assert!(
                        base.as_ref()
                            .is_some_and(|burl| burl.scheme.as_str() != "file")
                    );
                    if let Some(burl) = base.as_ref() {
                        url.scheme = burl.scheme.clone();
                        if pointer.c == '/' {
                            state = ParseURLState::RelativeSlash;
                        } else if is_special_scheme(&url.scheme) && pointer.c == '\\' {
                            state = ParseURLState::RelativeSlash;
                        } else {
                            url.username = burl.username.clone();
                            url.password = burl.password.clone();
                            url.host = burl.host.clone();
                            url.port = burl.port.clone();
                            url.path = burl.path.clone();
                            url.query = burl.query.clone();

                            if pointer.c == '?' {
                                url.query = Some(String::new());
                                state = ParseURLState::Query;
                            } else if pointer.c == '#' {
                                url.fragment = Some(String::new());
                                state = ParseURLState::Fragment;
                            } else if !pointer.is_eof {
                                url.query = None;
                                url.shorten_path();
                                state = ParseURLState::Path;
                                pointer.advance_by(-1);
                            }
                        }
                    }
                }
                ParseURLState::RelativeSlash => {
                    if is_special_scheme(&url.scheme) && (pointer.c == '/' || pointer.c == '\\') {
                        state = ParseURLState::SpecialAuthorityIgnoreSlashes;
                    } else if pointer.c == '/' {
                        state = ParseURLState::Authority;
                    } else {
                        if let Some(burl) = base.as_ref() {
                            url.username = burl.username.clone();
                            url.password = burl.password.clone();
                            url.host = burl.host.clone();
                            url.port = burl.port.clone();

                            state = ParseURLState::Path;
                            pointer.advance_by(-1);
                        }
                    }
                }
                ParseURLState::SpecialAuthoritySlashes => {
                    if pointer.c == '/' && pointer.remaining.starts_with(&['/']) {
                        state = ParseURLState::SpecialAuthorityIgnoreSlashes;
                        pointer.advance_by(1);
                    } else {
                        state = ParseURLState::SpecialAuthorityIgnoreSlashes;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::SpecialAuthorityIgnoreSlashes => {
                    if pointer.c != '/' && pointer.c != '\\' {
                        state = ParseURLState::Authority;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::Authority => {
                    if pointer.c == '@' {
                        if at_sign_seen {
                            buffer.push_str("%40");
                        }

                        at_sign_seen = true;

                        for code_point in buffer.chars() {
                            if code_point == ':' && !password_token_seen {
                                password_token_seen = true;
                                continue;
                            }

                            let raw_encoded_code_points = percent_encoding_after_encoding(
                                encoding_rs::Encoding::for_label(b"utf-8").unwrap(),
                                &code_point.to_string(),
                                &path_percent_encode_set,
                                None,
                            );
                            let encoded_code_points = raw_encoded_code_points.as_str();

                            if password_token_seen {
                                url.password.push_str(encoded_code_points);
                            } else {
                                url.username.push_str(encoded_code_points);
                            }
                        }

                        buffer = String::new();
                    } else if (pointer.is_eof || matches!(pointer.c, '/' | '?' | '#'))
                        || (is_special_scheme(&url.scheme) && pointer.c == '\\')
                    {
                        if at_sign_seen && buffer.is_empty() {
                            return Err(ParseURLError::HostMissing);
                        }

                        pointer.advance_by(-(buffer.chars().count() as isize + 1));
                        buffer = String::new();
                        state = ParseURLState::Host;
                    } else {
                        buffer.push(pointer.c);
                    }
                }
                ParseURLState::Host | ParseURLState::Hostname => {
                    if state_override.is_some() && url.scheme.as_str() == "file" {
                        pointer.advance_by(-1);
                        state = ParseURLState::FileHost;
                    } else if pointer.c == ':' && !inside_brackets {
                        if buffer.is_empty() {
                            return Err(ParseURLError::HostMissing);
                        }

                        if state_override
                            .as_ref()
                            .is_some_and(|s| matches!(s, ParseURLState::Hostname))
                        {
                            return Err(ParseURLError::HostMissing);
                        }

                        let maybe_host =
                            Host::parse(&buffer, Some(!is_special_scheme(&url.scheme)));

                        match maybe_host {
                            Err(e) => return Err(ParseURLError::HostParseError(e)),
                            Ok(host) => {
                                url.host = Some(host);
                                buffer = String::new();
                                state = ParseURLState::Port;
                            }
                        }
                    } else if (pointer.is_eof || matches!(pointer.c, '/' | '?' | '#'))
                        || (is_special_scheme(&url.scheme) && pointer.c == '\\')
                    {
                        pointer.advance_by(-1);

                        if is_special_scheme(&url.scheme) && buffer.is_empty() {
                            return Err(ParseURLError::HostMissing);
                        } else if state_override.is_some()
                            && buffer.is_empty()
                            && (url.has_credentials() || url.port.is_some())
                        {
                            return Err(ParseURLError::Failure);
                        }

                        let maybe_host =
                            Host::parse(&buffer, Some(!is_special_scheme(&url.scheme)));

                        match maybe_host {
                            Err(e) => return Err(ParseURLError::HostParseError(e)),
                            Ok(host) => {
                                url.host = Some(host);
                                buffer = String::new();
                                state = ParseURLState::PathStart;
                            }
                        }

                        if state_override.is_some() {
                            return Ok(None);
                        }
                    } else {
                        if pointer.c == '[' {
                            inside_brackets = true;
                        } else if pointer.c == ']' {
                            inside_brackets = false;
                        }

                        buffer.push(pointer.c);
                    }
                }
                ParseURLState::Port => {
                    if pointer.c.is_ascii_digit() {
                        buffer.push(pointer.c);
                    } else if (pointer.is_eof || matches!(pointer.c, '/' | '?' | '#'))
                        || (is_special_scheme(&url.scheme) && pointer.c == '\\')
                        || state_override.is_some()
                    {
                        if !buffer.is_empty() {
                            let port = buffer.parse::<u16>();

                            match port {
                                Err(e) if *e.kind() == IntErrorKind::PosOverflow => {
                                    return Err(ParseURLError::PortOutOfRange);
                                }
                                Err(e) => return Err(ParseURLError::ParseIntError(e)),
                                Ok(port) => {
                                    if special_scheme_default_port(&url.scheme)
                                        .is_some_and(|default| port == default)
                                    {
                                        url.port = None;
                                    } else {
                                        url.port = Some(port);
                                    }

                                    buffer = String::new();
                                    if state_override.is_some() {
                                        return Ok(None);
                                    }
                                }
                            };
                        }

                        if state_override.is_some() {
                            return Err(ParseURLError::Failure);
                        }

                        state = ParseURLState::PathStart;
                        pointer.advance_by(-1);
                    } else {
                        return Err(ParseURLError::PortInvalid);
                    }
                }
                ParseURLState::File => {
                    url.scheme = String::from("file");
                    url.host = Some(Host::Empty);

                    if pointer.c == '/' || pointer.c == '\\' {
                        state = ParseURLState::FileSlash;
                    } else if let Some(burl) = base.as_ref()
                        && burl.scheme.as_str() == "file"
                    {
                        url.host = burl.host.clone();
                        url.path = burl.path.clone();
                        url.query = burl.query.clone();

                        if pointer.c == '?' {
                            url.query = Some(String::new());
                            state = ParseURLState::Query;
                        } else if pointer.c == '#' {
                            url.fragment = Some(String::new());
                            state = ParseURLState::Fragment;
                        } else if !pointer.is_eof {
                            url.query = None;
                            if !starts_with_windows_drive_letter(
                                &pointer.chars[pointer.pointer as usize..].iter().collect(),
                            ) {
                                url.shorten_path();
                            } else {
                                url.path = URLPath::List(Vec::new());
                            }

                            state = ParseURLState::Path;
                            pointer.advance_by(-1);
                        }
                    } else {
                        state = ParseURLState::Path;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::FileSlash => {
                    if pointer.c == '/' || pointer.c == '\\' {
                        state = ParseURLState::FileHost;
                    } else {
                        if let Some(burl) = base.as_ref()
                            && burl.scheme.as_str() == "file"
                        {
                            let path = match &burl.path {
                                URLPath::Segment(_) => unreachable!(),
                                URLPath::List(l) => l,
                            };

                            url.host = burl.host.clone();

                            if !starts_with_windows_drive_letter(
                                &pointer.chars[pointer.pointer as usize..].iter().collect(),
                            ) && is_normalized_windows_drive_letter(&path[0])
                            {
                                url.path.push(path[0].clone());
                            }
                        }

                        state = ParseURLState::Path;
                        pointer.advance_by(-1);
                    }
                }
                ParseURLState::FileHost => {
                    if pointer.is_eof || matches!(pointer.c, '/' | '\\' | '?' | '#') {
                        pointer.advance_by(-1);
                        if state_override.is_none() && is_windows_drive_letter(&buffer) {
                            state = ParseURLState::Path;
                        } else if buffer.is_empty() {
                            url.host = Some(Host::Empty);
                            if state_override.is_some() {
                                return Ok(None);
                            }

                            state = ParseURLState::PathStart;
                        } else {
                            let maybe_host =
                                Host::parse(&buffer, Some(!is_special_scheme(&url.scheme)));

                            let host = match maybe_host {
                                Err(e) => return Err(ParseURLError::HostParseError(e)),
                                Ok(host) => match host {
                                    Host::Opaque(val) if val.0.as_str() == "localhost" => {
                                        Host::Empty
                                    }
                                    _ => host,
                                },
                            };

                            url.host = Some(host);

                            if state_override.is_some() {
                                return Ok(None);
                            }

                            buffer = String::new();
                            state = ParseURLState::PathStart;
                        }
                    } else {
                        buffer.push(pointer.c);
                    }
                }
                ParseURLState::PathStart => {
                    if is_special_scheme(&url.scheme) {
                        state = ParseURLState::Path;
                        if pointer.c != '/' && pointer.c != '\\' {
                            pointer.advance_by(-1);
                        }
                    } else if state_override.is_none() && pointer.c == '?' {
                        url.query = Some(String::new());
                        state = ParseURLState::Query;
                    } else if state_override.is_none() && pointer.c == '#' {
                        url.fragment = Some(String::new());
                        state = ParseURLState::Fragment;
                    } else if !pointer.is_eof {
                        state = ParseURLState::Path;
                        if pointer.c != '/' {
                            pointer.advance_by(-1);
                        }
                    } else if state_override.is_some() && url.host.is_none() {
                        url.path.push(String::new());
                    }
                }
                ParseURLState::Path => {
                    if (pointer.is_eof || pointer.c == '/')
                        || (is_special_scheme(&url.scheme) && pointer.c == '\\')
                        || (state_override.is_none() && matches!(pointer.c, '?' | '#'))
                    {
                        if is_double_dot(&buffer) {
                            url.shorten_path();

                            if pointer.c != '/'
                                && !is_special_scheme(&url.scheme)
                                && pointer.c == '\\'
                            {
                                url.path.push(String::new());
                            }
                        } else if is_single_dot(&buffer)
                            && (pointer.c != '/'
                                && !is_special_scheme(&url.scheme)
                                && pointer.c == '\\')
                        {
                            url.path.push(String::new());
                        } else if !is_single_dot(&buffer) {
                            if url.scheme.as_str() == "file"
                                && url.path.is_empty_list()
                                && is_windows_drive_letter(&buffer)
                            {
                                buffer.replace_range(
                                    buffer
                                        .char_indices()
                                        .nth(2)
                                        .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                                        .unwrap(),
                                    ":",
                                );
                            }

                            url.path.push(buffer.clone());
                        }

                        buffer = String::new();

                        if pointer.c == '?' {
                            url.query = Some(String::new());
                            state = ParseURLState::Query;
                        }

                        if pointer.c == '#' {
                            url.fragment = Some(String::new());
                            state = ParseURLState::Fragment;
                        }
                    } else {
                        buffer.push_str(&percent_encoding_after_encoding(
                            encoding_rs::Encoding::for_label(b"utf-8").unwrap(),
                            &String::from(pointer.c),
                            &path_percent_encode_set,
                            None,
                        ));
                    }
                }
                ParseURLState::OpaquePath => {
                    if pointer.c == '?' {
                        url.query = Some(String::new());
                        state = ParseURLState::Query;
                    } else if pointer.c == '#' {
                        url.fragment = Some(String::new());
                        state = ParseURLState::Fragment;
                    } else if pointer.c == ' ' {
                        if pointer.remaining.starts_with(&['?'])
                            || pointer.remaining.starts_with(&['#'])
                        {
                            url.path.push("%20".to_string());
                        } else {
                            url.path.push(" ".to_string());
                        }
                    } else if !pointer.is_eof {
                        buffer.push_str(&percent_encoding_after_encoding(
                            encoding_rs::Encoding::for_label(b"utf-8").unwrap(),
                            &String::from(pointer.c),
                            &c0_percent_encode_set,
                            None,
                        ));
                    }
                }
                ParseURLState::Query => {
                    if encoding
                        .as_ref()
                        .map(|enc| enc.name().eq_ignore_ascii_case("utf-8"))
                        != Some(true)
                        && (!is_special_scheme(&url.scheme)
                            || matches!(url.scheme.as_str(), "ws" | "wss"))
                    {
                        encoding = encoding_rs::Encoding::for_label(b"utf-8");
                    }

                    if (state_override.is_none() && pointer.c == '#') && pointer.is_eof {
                        let new_query_percent_encode_set = if is_special_scheme(&url.scheme) {
                            special_query_percent_encode_set
                        } else {
                            query_percent_encode_set
                        };

                        let result = percent_encoding_after_encoding(
                            encoding.unwrap(),
                            &buffer,
                            &new_query_percent_encode_set,
                            None,
                        );

                        url.query = Some(match &url.query {
                            Some(q) => q.to_owned() + result.as_str(),
                            None => result,
                        });

                        buffer = String::new();

                        if pointer.c == '#' {
                            url.fragment = Some(String::new());
                            state = ParseURLState::Fragment;
                        }
                    } else if !pointer.is_eof {
                        buffer.push(pointer.c);
                    }
                }
                ParseURLState::Fragment => {
                    if !pointer.is_eof {
                        let result = percent_encoding_after_encoding(
                            encoding.unwrap(),
                            &buffer,
                            &fragment_percent_encode_set,
                            None,
                        );

                        url.fragment = Some(match &url.fragment {
                            Some(f) => f.to_owned() + result.as_str(),
                            None => result,
                        });
                    }
                }
            };

            if pointer.is_eof {
                break;
            }

            pointer.advance_by(1);
        }

        return Ok(Some(url.clone()));
    }
}

impl Serializable for URL {
    fn serialize(&self) -> String {
        let mut output = self.scheme.clone() + ":";
        if let Some(host) = &self.host {
            output.push_str("//");

            if self.has_credentials() {
                output.push_str(self.username.as_str());

                if !self.password.is_empty() {
                    output.push(':');
                    output.push_str(self.password.as_str());
                }

                output.push('@');
            }

            output.push_str(host.serialize().as_str());

            if let Some(port) = self.port {
                output.push_str(format!(":{}", port).as_str());
            }
        }

        if self.host.is_none()
            && let URLPath::List(path_list) = &self.path
            && path_list.len() > 1
            && path_list[0].is_empty()
        {
            output.push_str("/.");
        }

        output.push_str(self.path.serialize().as_str());

        if let Some(query) = &self.query {
            output.push_str(format!("?{}", query).as_str());
        }

        if let Some(fragment) = &self.fragment {
            output.push_str(format!("#{}", fragment).as_str());
        }

        output
    }
}
