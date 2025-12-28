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

pub struct IPv4(u32);
pub struct IPv6([u16; 8]);

pub enum IPAddress {
    IPv4(IPv4),
    IPv6(IPv6),
}

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
                        println!("number: {}", number);
                        println!(
                            "chars: {}",
                            pointer.chars.clone().into_iter().collect::<String>()
                        );
                        println!("position: {}", pointer.pointer);
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

                    println!("piece: {}", ipv4_piece.unwrap());

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

pub enum Host {
    Domain(String),
    IPAddress(IPAddress),
    Opaque(String),
    Empty,
}

pub enum HostParseError {
    IPv6UnclosedValidation,
}

impl Host {
    pub fn parse(input: String, is_opaque: Option<bool>) -> Result<Host, HostParseError> {
        if input.starts_with('[') {
            if !input.ends_with(']') {
                return Err(HostParseError::IPv6UnclosedValidation);
            }
        }

        todo!()
    }
}

pub struct SpecURL {
    scheme: String,
    username: String,
    password: String,
    host: Option<Host>,
}
