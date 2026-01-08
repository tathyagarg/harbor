use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::TableTrait;

fn flags_to_string(flags: uint16) -> String {
    let mut flag_descriptions = Vec::new();

    if flags & 0x0001 != 0 {
        flag_descriptions.push("Baseline for font at y=0");
    }
    if flags & 0x0002 != 0 {
        flag_descriptions.push("Left sidebearing point at x=0");
    }
    if flags & 0x0004 != 0 {
        flag_descriptions.push("Instructions may depend on point size");
    }
    if flags & 0x0008 != 0 {
        flag_descriptions.push("Force ppem to integer values for all internal scaler math");
    }
    if flags & 0x0010 != 0 {
        flag_descriptions.push("Instructions may alter advance width");
    }
    if flags & 0x0020 != 0 {
        flag_descriptions.push("Font converted to CFF format");
    }
    if flags & 0x0040 != 0 {
        flag_descriptions.push("Optimized for ClearType");
    }
    if flags & 0x0080 != 0 {
        flag_descriptions.push("Last resort font");
    }

    flag_descriptions.join(", ")
}

fn mac_style_to_string(mac_style: uint16) -> String {
    let mut styles = Vec::new();

    if mac_style & 0x0001 != 0 {
        styles.push("Bold");
    }
    if mac_style & 0x0002 != 0 {
        styles.push("Italic");
    }
    if mac_style & 0x0004 != 0 {
        styles.push("Underline");
    }
    if mac_style & 0x0008 != 0 {
        styles.push("Outline");
    }
    if mac_style & 0x0010 != 0 {
        styles.push("Shadow");
    }
    if mac_style & 0x0020 != 0 {
        styles.push("Condensed");
    }
    if mac_style & 0x0040 != 0 {
        styles.push("Extended");
    }

    styles.join(", ")
}

#[derive(Clone, Default)]
pub struct HeaderTable {
    /// Major version number of the font header table — set to 1.
    pub major_version: uint16,

    /// Minor version number of the font header table — set to 0.
    pub minor_version: uint16,

    /// Set by font manufacturer.
    pub font_revision: Fixed,

    /// To compute: set it to 0, sum the entire font as uint32, then store 0xB1B0AFBA - sum.
    /// If the font is used as a component in a font collection file, the value of this field
    /// will be invalidated by changes to the file structure and font table directory, and must
    /// be ignored.
    pub check_sum_adjustment: uint32,
    // Set to 0x5F0F3CF5.
    // pub magic_number: uint32,
    /// Flags
    pub flags: uint16,

    /// Set to a value from 16 to 16384. Any value in this range is valid. In fonts that have
    /// TrueType outlines, a power of 2 is recommended as this allows performance optimization
    /// in some rasterizers.
    pub units_per_em: uint16,

    /// Number of seconds since 12:00 midnight that started January 1st, 1904, in GMT/UTC time zone.
    pub created: LongDateTime,
    pub modified: LongDateTime,

    /// Minimum x coordinate across all glyph bounding boxes.
    pub x_min: int16,

    /// Minimum y coordinate across all glyph bounding boxes.
    pub y_min: int16,

    /// Maximum x coordinate across all glyph bounding boxes.
    pub x_max: int16,

    /// Maximum y coordinate across all glyph bounding boxes.
    pub y_max: int16,

    /// Mac style flags.
    pub mac_style: uint16,

    /// Smallest readable size in pixels.
    pub lowest_rec_ppem: uint16,

    /// Deprecated (Set to 2).
    #[deprecated]
    pub font_direction_hint: int16,

    /// 0 for short offsets (Offset16), 1 for long (Offset32).
    pub index_to_loc_format: int16,
}

impl Debug for HeaderTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeaderTable")
            .field("major_version", &self.major_version)
            .field("minor_version", &self.minor_version)
            .field("font_revision", &fixed_to_string(self.font_revision))
            .field(
                "check_sum_adjustment",
                &format_args!("0x{:08X}", self.check_sum_adjustment),
            )
            .field("flags", &flags_to_string(self.flags))
            .field("flags_raw", &self.flags)
            .field("units_per_em", &self.units_per_em)
            .field("created", &self.created)
            .field("modified", &self.modified)
            .field("x_min", &self.x_min)
            .field("y_min", &self.y_min)
            .field("x_max", &self.x_max)
            .field("y_max", &self.y_max)
            .field("mac_style", &mac_style_to_string(self.mac_style))
            .field("mac_style_raw", &self.mac_style)
            .field("lowest_rec_ppem", &self.lowest_rec_ppem)
            .field("font_direction_hint", &self.font_direction_hint)
            .field("index_to_loc_format", &self.index_to_loc_format)
            .finish()
    }
}

impl TableTrait for HeaderTable {
    fn parse(data: &[u8]) -> HeaderTable {
        HeaderTable {
            major_version: uint16::from_be_bytes(data[0..2].try_into().unwrap()),
            minor_version: uint16::from_be_bytes(data[2..4].try_into().unwrap()),
            font_revision: Fixed::from_be_bytes(data[4..8].try_into().unwrap()),
            check_sum_adjustment: uint32::from_be_bytes(data[8..12].try_into().unwrap()),
            // magic number consumes 4 bytes
            flags: uint16::from_be_bytes(data[16..18].try_into().unwrap()),
            units_per_em: uint16::from_be_bytes(data[18..20].try_into().unwrap()),
            created: LongDateTime::from_be_bytes(data[20..28].try_into().unwrap()),
            modified: LongDateTime::from_be_bytes(data[28..36].try_into().unwrap()),
            x_min: int16::from_be_bytes(data[36..38].try_into().unwrap()),
            y_min: int16::from_be_bytes(data[38..40].try_into().unwrap()),
            x_max: int16::from_be_bytes(data[40..42].try_into().unwrap()),
            y_max: int16::from_be_bytes(data[42..44].try_into().unwrap()),
            mac_style: uint16::from_be_bytes(data[44..46].try_into().unwrap()),
            lowest_rec_ppem: uint16::from_be_bytes(data[46..48].try_into().unwrap()),
            font_direction_hint: int16::from_be_bytes(data[48..50].try_into().unwrap()),
            index_to_loc_format: int16::from_be_bytes(data[50..52].try_into().unwrap()),
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("HeaderTable does not required construction - simply use HeaderTable::parse()");
    }
}

impl HeaderTable {
    pub fn new() -> HeaderTable {
        HeaderTable::default()
    }
}
