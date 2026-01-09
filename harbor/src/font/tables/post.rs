#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

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
}
