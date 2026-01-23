#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

pub const MACINTOSH_STANDARD_GLYPH_NAMES: [&str; 258] = [
    ".notdef",
    ".null",
    "nonmarkingreturn",
    "space",
    "exclam",
    "quotedbl",
    "numbersign",
    "dollar",
    "percent",
    "ampersand",
    "quotesingle",
    "parenleft",
    "parenright",
    "asterisk",
    "plus",
    "comma",
    "hyphen",
    "period",
    "slash",
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "colon",
    "semicolon",
    "less",
    "equal",
    "greater",
    "question",
    "at",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "bracketleft",
    "backslash",
    "bracketright",
    "asciicircum",
    "underscore",
    "grave",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "braceleft",
    "bar",
    "braceright",
    "asciitilde",
    "Adieresis",
    "Aring",
    "Ccedilla",
    "Eacute",
    "Ntilde",
    "Odieresis",
    "Udieresis",
    "aacute",
    "agrave",
    "acircumflex",
    "adieresis",
    "atilde",
    "aring",
    "ccedilla",
    "eacute",
    "egrave",
    "ecircumflex",
    "edieresis",
    "iacute",
    "igrave",
    "icircumflex",
    "idieresis",
    "ntilde",
    "oacute",
    "ograve",
    "ocircumflex",
    "odieresis",
    "otilde",
    "uacute",
    "ugrave",
    "ucircumflex",
    "udieresis",
    "dagger",
    "degree",
    "cent",
    "sterling",
    "section",
    "bullet",
    "paragraph",
    "germandbls",
    "registered",
    "copyright",
    "trademark",
    "acute",
    "dieresis",
    "notequal",
    "AE",
    "Oslash",
    "infinity",
    "plusminus",
    "lessequal",
    "greaterequal",
    "yen",
    "mu",
    "partialdiff",
    "summation",
    "product",
    "pi",
    "integral",
    "ordfeminine",
    "ordmasculine",
    "Omega",
    "ae",
    "oslash",
    "questiondown",
    "exclamdown",
    "logicalnot",
    "radical",
    "florin",
    "approxequal",
    "Delta",
    "guillemotleft",
    "guillemotright",
    "ellipsis",
    "nonbreakingspace",
    "Agrave",
    "Atilde",
    "Otilde",
    "OE",
    "oe",
    "endash",
    "emdash",
    "quotedblleft",
    "quotedblright",
    "quoteleft",
    "quoteright",
    "divide",
    "lozenge",
    "ydieresis",
    "Ydieresis",
    "fraction",
    "currency",
    "guilsinglleft",
    "guilsinglright",
    "fi",
    "fl",
    "daggerdbl",
    "periodcentered",
    "quotesinglbase",
    "quotedblbase",
    "perthousand",
    "Acircumflex",
    "Ecircumflex",
    "Aacute",
    "Edieresis",
    "Egrave",
    "Iacute",
    "Icircumflex",
    "Idieresis",
    "Igrave",
    "Oacute",
    "Ocircumflex",
    "apple",
    "Ograve",
    "Uacute",
    "Ucircumflex",
    "Ugrave",
    "dotlessi",
    "circumflex",
    "tilde",
    "macron",
    "breve",
    "dotaccent",
    "ring",
    "cedilla",
    "hungarumlaut",
    "ogonek",
    "caron",
    "Lslash",
    "lslash",
    "Scaron",
    "scaron",
    "Zcaron",
    "zcaron",
    "brokenbar",
    "Eth",
    "eth",
    "Yacute",
    "yacute",
    "Thorn",
    "thorn",
    "minus",
    "multiply",
    "onesuperior",
    "twosuperior",
    "threesuperior",
    "onehalf",
    "onequarter",
    "threequarters",
    "franc",
    "Gbreve",
    "gbreve",
    "Idotaccent",
    "Scedilla",
    "scedilla",
    "Cacute",
    "cacute",
    "Ccaron",
    "ccaron",
    "dcroat",
];

#[derive(Clone)]
pub struct PostTable_VersionSpecific_V2_0 {
    /// Number of glyphs (this should be the same as numGlyphs in 'maxp' table).
    pub num_glyphs: uint16,

    /// Array of indices into the string data.
    pub glyph_name_indices: Vec<uint16>,

    /// Storage for the string data
    pub string_data: Vec<u8>,

    pub _glyph_names: Vec<String>,
}

impl Debug for PostTable_VersionSpecific_V2_0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostTable_VersionSpecific_V2_0")
            .field("num_glyphs", &self.num_glyphs)
            .field(
                "glyph_name_indices",
                &format!("[{} entries]", self.glyph_name_indices.len()),
            )
            .field(
                "glyph_name_indices_preview",
                &&self.glyph_name_indices[..std::cmp::min(4, self.glyph_name_indices.len())],
            )
            .field(
                "string_data",
                &format!("[{} bytes]", self.string_data.len()),
            )
            .field(
                "glyph_names",
                &format!("[{} entries]", self._glyph_names.len()),
            )
            .field(
                "glyph_names_preview",
                &&self._glyph_names[..std::cmp::min(4, self._glyph_names.len())],
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum PostTable_VersionSpecific {
    Version_1_0,
    Version_2_0(PostTable_VersionSpecific_V2_0),

    #[deprecated]
    Version_2_5 {
        /// Number of glyphs.
        num_glyphs: uint16,

        /// Difference between the glyph index and the standard order of the glyph.
        glyph_name_offsets: Vec<int8>,
    },

    Version_3_0,
}

#[derive(Clone, Debug)]
pub struct PostTable {
    pub version: Version16Dot16,

    /// Italic angle in counter-clockwise degrees from the vertical. Zero for upright text,
    /// negative for text that leans to the right (forward).
    pub italic_angle: Fixed,

    /// Suggested y-coordinate of the top of the underline.
    pub underline_position: FWORD,

    /// Suggested values for the underline thickness. In general, the underline thickness should
    /// match the thickness of the underscore character (U+005F LOW LINE), and should also match
    /// the strikeout thickness, which is specified in the OS/2 table.
    pub underline_thickness: FWORD,

    /// Set to 0 if the font is proportionally spaced, non-zero if the font is not proportionally
    /// spaced (i.e., monospaced).
    pub is_fixed_pitch: uint32,

    /// Minimum memory usage when an OpenType font is downloaded.
    pub min_mem_type42: uint32,

    /// Maximum memory usage when an OpenType font is downloaded.
    pub max_mem_type42: uint32,

    /// Minimum memory usage when an OpenType font is downloaded as a Type 1 font.
    pub min_mem_type1: uint32,

    /// Maximum memory usage when an OpenType font is downloaded as a Type 1 font.
    pub max_mem_type1: uint32,

    _version_specific: Option<PostTable_VersionSpecific>,
}

impl TableTrait for PostTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> PostTable {
        let version = Version16Dot16::from_be_bytes(data[0..4].try_into().unwrap());
        let italic_angle = Fixed::from_be_bytes(data[4..8].try_into().unwrap());
        let underline_position = FWORD::from_be_bytes(data[8..10].try_into().unwrap());
        let underline_thickness = FWORD::from_be_bytes(data[10..12].try_into().unwrap());
        let is_fixed_pitch = uint32::from_be_bytes(data[12..16].try_into().unwrap());
        let min_mem_type42 = uint32::from_be_bytes(data[16..20].try_into().unwrap());
        let max_mem_type42 = uint32::from_be_bytes(data[20..24].try_into().unwrap());
        let min_mem_type1 = uint32::from_be_bytes(data[24..28].try_into().unwrap());
        let max_mem_type1 = uint32::from_be_bytes(data[28..32].try_into().unwrap());

        PostTable {
            version,
            italic_angle,
            underline_position,
            underline_thickness,
            is_fixed_pitch,
            min_mem_type42,
            max_mem_type42,
            min_mem_type1,
            max_mem_type1,
            _version_specific: PostTable::parse_version_specific(version, data),
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("PostTable does not require construction - simply use PostTable::parse()");
    }
}

impl PostTable {
    fn parse_version_specific(
        version: Version16Dot16,
        data: &[u8],
    ) -> Option<PostTable_VersionSpecific> {
        match version {
            0x00010000 => Some(PostTable_VersionSpecific::Version_1_0),
            0x00020000 => {
                let mut offset = 32;

                let mut version_specific_data = PostTable_VersionSpecific_V2_0 {
                    num_glyphs: 0,
                    glyph_name_indices: Vec::new(),
                    string_data: Vec::new(),
                    _glyph_names: Vec::new(),
                };

                version_specific_data.num_glyphs =
                    uint16::from_be_bytes(data[offset..offset + 2].try_into().unwrap());
                offset += 2;

                version_specific_data.glyph_name_indices =
                    Vec::with_capacity(version_specific_data.num_glyphs as usize);

                version_specific_data._glyph_names =
                    Vec::with_capacity(version_specific_data.num_glyphs as usize);

                for _ in 0..version_specific_data.num_glyphs {
                    let glyph_index =
                        uint16::from_be_bytes(data[offset..offset + 2].try_into().unwrap());
                    version_specific_data.glyph_name_indices.push(glyph_index);
                    offset += 2;
                }

                let start_offset = offset;

                // The glyph names for indices 0-257 are predefined and not stored here.
                // We only need to read the custom glyph names starting from index 258.
                for _ in 0..version_specific_data.num_glyphs - 258 {
                    if offset >= data.len() {
                        break;
                    }

                    let string_length = data[offset] as usize;
                    offset += 1;

                    let _glyph_name =
                        String::from_utf8(data[offset..offset + string_length].to_vec()).unwrap();
                    offset += string_length;

                    version_specific_data._glyph_names.push(_glyph_name);
                }

                version_specific_data.string_data = data[start_offset..offset].to_vec();

                Some(PostTable_VersionSpecific::Version_2_0(
                    version_specific_data,
                ))
            }
            0x00025000 => None,
            0x00030000 => Some(PostTable_VersionSpecific::Version_3_0),
            _ => None,
        }
    }

    pub fn glyph_name(&self, glyph_index: uint16) -> Option<String> {
        if let Some(PostTable_VersionSpecific::Version_2_0(v2_0)) = &self._version_specific {
            if glyph_index < v2_0.num_glyphs {
                let name_index = v2_0.glyph_name_indices[glyph_index as usize];

                if name_index < 258 {
                    // Predefined glyph names
                    return Some(MACINTOSH_STANDARD_GLYPH_NAMES[name_index as usize].to_string());
                } else {
                    // Custom glyph names
                    let custom_index = name_index - 258;
                    if (custom_index as usize) < v2_0._glyph_names.len() {
                        return Some(v2_0._glyph_names[custom_index as usize].clone());
                    }
                }
            }
        }
        None
    }
}
