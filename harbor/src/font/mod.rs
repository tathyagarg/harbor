pub mod ttc;
pub mod ttf;

pub mod tables;

pub use ttc::parse_ttc;

pub fn parse_ttf(data: &[u8]) -> ttf::TableDirectory {
    ttf::parse_table_directory(data, None)
}

pub mod otf_dtypes {
    #![allow(non_camel_case_types)]

    use std::str::Utf8Error;

    pub type GLYPH_ID = u16;

    pub type uint8 = u8;
    pub type int8 = i8;
    pub type uint16 = u16;
    pub type int16 = i16;

    // TODO: Determine usage and workaround
    // type uint24

    pub type uint32 = u32;
    pub type int32 = i32;

    // TODO: Determine usage and workaround
    pub type Fixed = int32;

    pub type FWORD = int16;
    pub type UFWORD = uint16;

    pub type F2DOT14 = uint16;

    pub fn f2dot14_to_f32(value: F2DOT14) -> f32 {
        let sign = if (value & 0x8000) != 0 { -1.0 } else { 1.0 };

        let integer_part = (value >> 14 & 1) as f32 * sign;
        let fractional_part = (value & 0x3FFF) as f32 / 16384.0;
        integer_part + fractional_part
    }

    pub type LongDateTime = i64;

    pub type Tag = [uint8; 4];

    /// Each byte within the array must have a value in the range 0x20 to 0x7E.
    /// This corresponds to the range of values of Unicode Basic Latin characters in UTF-8
    /// encoding, which is the same as the printable ASCII characters.
    /// ...
    /// It must have one to four non-space characters, padded with trailing spaces (byte value 0x20).
    /// A space character must not be followed by a non-space character.
    pub fn is_valid_tag(t: Tag) -> bool {
        if t.into_iter().any(|n| !(0x20..=0x7E).contains(&n)) {
            return false;
        }

        let non_space_count = t.into_iter().filter(|n| *n != 0x20).count();

        // How would non_space_count > 4 even happen
        if non_space_count < 1 || non_space_count > 4 {
            return false;
        }

        let mut space_seen = false;
        for n in t.iter() {
            if *n == 0x20 {
                space_seen = true;
            } else if space_seen {
                // Non space character after space seen
                return false;
            }
        }

        true
    }

    pub fn tag_as_str(t: &Tag) -> Result<String, Utf8Error> {
        str::from_utf8(t).map(|s| s.to_string())
    }

    #[allow(dead_code)]
    pub type Offset8 = uint8;
    pub type Offset16 = uint16;

    // TODO: Determine usage and workaround
    // type Offset24 = uint24;

    pub type Offset32 = uint32;

    pub type Version16Dot16 = int32;

    pub trait FromBeBytes {
        fn from_data(bytes: &[u8]) -> Self;
    }

    impl FromBeBytes for uint16 {
        fn from_data(bytes: &[u8]) -> Self {
            uint16::from_be_bytes(bytes[..2].try_into().unwrap())
        }
    }

    impl FromBeBytes for uint32 {
        fn from_data(bytes: &[u8]) -> Self {
            uint32::from_be_bytes(bytes[..4].try_into().unwrap())
        }
    }

    impl FromBeBytes for int16 {
        fn from_data(bytes: &[u8]) -> Self {
            int16::from_be_bytes(bytes[..2].try_into().unwrap())
        }
    }

    pub fn fixed_to_string(fixed: Fixed) -> String {
        let integer_part = fixed >> 16;
        let fractional_part = fixed & 0xFFFF;
        format!("{}.{}", integer_part, fractional_part)
    }
}
