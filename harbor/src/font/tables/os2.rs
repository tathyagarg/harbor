#![allow(non_camel_case_types)]

use std::fmt::Debug;
use std::ops::BitAnd;

use crate::font::otf_dtypes::*;
use crate::font::tables::head::MacStyle;
use crate::font::tables::{ParseContext, TableTrait};

fn weight_to_string(weight: uint16) -> &'static str {
    match weight {
        100 => "Thin",
        200 => "Extra Light",
        300 => "Light",
        400 => "Normal",
        500 => "Medium",
        600 => "Semi Bold",
        700 => "Bold",
        800 => "Extra Bold",
        900 => "Black",
        _ => "Unknown",
    }
}

fn width_to_string(width: uint16) -> &'static str {
    match width {
        1 => "Ultra Condensed",
        2 => "Extra Condensed",
        3 => "Condensed",
        4 => "Semi Condensed",
        5 => "Medium (Normal)",
        6 => "Semi Expanded",
        7 => "Expanded",
        8 => "Extra Expanded",
        9 => "Ultra Expanded",
        _ => "Unknown",
    }
}

fn fstype_to_string(fstype: uint16) -> String {
    let mut flags = Vec::new();
    if fstype & 0x0001 != 0 {
        flags.push("Installable Embedding");
    }
    if fstype & 0x0002 != 0 {
        flags.push("Restricted License Embedding");
    }
    if fstype & 0x0004 != 0 {
        flags.push("Preview & Print Embedding");
    }
    if fstype & 0x0008 != 0 {
        flags.push("Editable Embedding");
    }
    if fstype & 0x0100 != 0 {
        flags.push("No Subsetting");
    }
    if fstype & 0x0200 != 0 {
        flags.push("Bitmap Embedding Only");
    }

    flags.join(", ")
}

fn panose_to_string(panose: &[uint8; 10]) -> String {
    let mut panose_str: [&str; 10] = ["Any"; 10];

    panose_str[0] = match panose[0] {
        0 => "Any",
        1 => "No Fit",
        2 => "Latin Text",
        3 => "Latin Hand Written",
        4 => "Latin Decorative",
        5 => "Latin Symbol",
        _ => "Unknown",
    };

    panose_str[2] = match panose[2] {
        0 => "Any",
        1 => "No Fit",
        2 => "Very Light",
        3 => "Light",
        4 => "Thin",
        5 => "Book",
        6 => "Medium",
        7 => "Demi",
        8 => "Bold",
        9 => "Heavy",
        10 => "Black",
        11 => "Extra Black",
        _ => "Unknown",
    };

    match panose[0] {
        2 => {
            panose_str[1] = match panose[1] {
                0 => "Any",
                1 => "No Fit",
                2 => "Cove",
                3 => "Obtuse Cove",
                4 => "Square Cove",
                5 => "Obtuse Square Cove",
                6 => "Square",
                7 => "Thin",
                8 => "Bone",
                9 => "Exaggerated",
                10 => "Triangle",
                11 => "Normal Sans",
                12 => "Obtuse Sans",
                13 => "Perpendicular Sans",
                14 => "Flared",
                15 => "Rounded",
                _ => "Unknown",
            };

            panose_str[3] = match panose[3] {
                0 => "Any",
                1 => "No Fit",
                2 => "Old Style",
                3 => "Modern",
                4 => "Even Width",
                5 => "Extended",
                6 => "Condensed",
                7 => "Very Extended",
                8 => "Very Condensed",
                9 => "Monospaced",
                _ => "Unknown",
            };

            panose_str[4] = match panose[4] {
                0 => "Any",
                1 => "No Fit",
                2 => "None",
                3 => "Very Low",
                4 => "Low",
                5 => "Medium Low",
                6 => "Medium",
                7 => "Medium High",
                8 => "High",
                9 => "Very High",
                _ => "Unknown",
            };

            panose_str[5] = match panose[5] {
                0 => "Any",
                1 => "No Fit",
                2 => "No Variation",
                3 => "Gradual/Diagonal",
                4 => "Gradual/Transitional",
                5 => "Gradual/Vertical",
                6 => "Gradual/Horizontal",
                7 => "Rapid/Vertical",
                8 => "Rapid/Horizontal",
                9 => "Instant/Vertical",
                10 => "Instant/Horizontal",
                _ => "Unknown",
            };

            panose_str[6] = match panose[6] {
                0 => "Any",
                1 => "No Fit",
                2 => "Straight Arms/Horizontal",
                3 => "Straight Arms/Wedge",
                4 => "Straight Arms/Vertical",
                5 => "Straight Arms/Single Serif",
                6 => "Straight Arms/Double Serif",
                7 => "Non-Straight Arms/Horizontal",
                8 => "Non-Straight Arms/Wedge",
                9 => "Non-Straight Arms/Vertical",
                10 => "Non-Straight Arms/Single Serif",
                11 => "Non-Straight Arms/Double Serif",
                _ => "Unknown",
            };

            panose_str[7] = match panose[7] {
                0 => "Any",
                1 => "No Fit",
                2 => "Normal/Contact",
                3 => "Normal/Weighted",
                4 => "Normal/Boxed",
                5 => "Normal/Flattened",
                6 => "Normal/Rounded",
                7 => "Normal/Off Center",
                8 => "Normal/Square",
                9 => "Oblique/Contact",
                10 => "Oblique/Weighted",
                11 => "Oblique/Boxed",
                12 => "Oblique/Flattened",
                13 => "Oblique/Rounded",
                14 => "Oblique/Off Center",
                15 => "Oblique/Square",
                _ => "Unknown",
            };

            panose_str[8] = match panose[8] {
                0 => "Any",
                1 => "No Fit",
                2 => "Straight/Trimmed",
                3 => "Straight/Pointed",
                4 => "Straight/Serifed",
                5 => "High/Trimmed",
                6 => "High/Pointed",
                7 => "High/Serifed",
                8 => "Constant/Trimmed",
                9 => "Constant/Pointed",
                10 => "Constant/Serifed",
                11 => "Low/Trimmed",
                12 => "Low/Pointed",
                13 => "Low/Serifed",
                _ => "Unknown",
            };

            panose_str[9] = match panose[9] {
                0 => "Any",
                1 => "No Fit",
                2 => "Constant/Small",
                3 => "Constant/Standard",
                4 => "Constant/Large",
                5 => "Ducking/Small",
                6 => "Ducking/Standard",
                7 => "Ducking/Large",
                _ => "Unknown",
            };
        }
        _ => {}
    }

    panose_str.join(", ")
}

#[repr(u16)]
pub enum FSSelectionFlags {
    Italic = 0x0001,
    Underscore = 0x0002,
    Negative = 0x0004,
    Outlined = 0x0008,
    Strikeout = 0x0010,
    Bold = 0x0020,
    Regular = 0x0040,
    UseTypoMetrics = 0x0080,
    WWS = 0x0100,
    Oblique = 0x0200,
}

impl BitAnd<FSSelectionFlags> for u16 {
    type Output = u16;

    fn bitand(self, rhs: FSSelectionFlags) -> Self::Output {
        self & (rhs as u16)
    }
}

fn fs_selection_to_string(fs_selection: uint16, mac_style: uint16) -> String {
    let mut flags = Vec::new();
    if fs_selection & FSSelectionFlags::Italic != 0 && mac_style & MacStyle::Italic != 0 {
        flags.push("Italic");
    }
    if fs_selection & FSSelectionFlags::Underscore != 0 {
        flags.push("Underscore");
    }
    if fs_selection & FSSelectionFlags::Negative != 0 {
        flags.push("Negative");
    }
    if fs_selection & FSSelectionFlags::Outlined != 0 {
        flags.push("Outlined");
    }
    if fs_selection & FSSelectionFlags::Strikeout != 0 {
        flags.push("Strikeout");
    }
    if fs_selection & FSSelectionFlags::Bold != 0 && mac_style & MacStyle::Bold != 0 {
        flags.push("Bold");
    }
    if fs_selection & FSSelectionFlags::Regular != 0 {
        flags.push("Regular");
    }
    if fs_selection & FSSelectionFlags::UseTypoMetrics != 0 {
        flags.push("Use Typo Metrics");
    }
    if fs_selection & FSSelectionFlags::WWS != 0 {
        flags.push("WWS");
    }
    if fs_selection & FSSelectionFlags::Oblique != 0 {
        flags.push("Oblique");
    }

    flags.join(", ")
}

// TODO: Custom implement Debug
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OS2Table_v5 {
    version: uint16,

    x_avg_char_width: FWORD,
    us_weight_class: uint16,
    us_width_class: uint16,
    fs_type: uint16,

    y_subscript_x_size: FWORD,
    y_subscript_y_size: FWORD,
    y_subscript_x_offset: FWORD,
    y_subscript_y_offset: FWORD,

    y_superscript_x_size: FWORD,
    y_superscript_y_size: FWORD,
    y_superscript_x_offset: FWORD,
    y_superscript_y_offset: FWORD,

    y_strikeout_size: FWORD,
    y_strikeout_position: FWORD,

    s_family_class: int16,

    panose: [uint8; 10],

    ul_unicode_range1: uint32,
    ul_unicode_range2: uint32,
    ul_unicode_range3: uint32,
    ul_unicode_range4: uint32,

    ach_vend_id: Tag,

    pub fs_selection: uint16,
    us_first_char_index: uint16,
    us_last_char_index: uint16,

    pub s_typo_ascender: FWORD,
    pub s_typo_descender: FWORD,
    pub s_typo_line_gap: FWORD,

    us_win_ascent: UFWORD,
    us_win_descent: UFWORD,

    ul_code_page_range1: uint32,
    ul_code_page_range2: uint32,

    sx_height: FWORD,
    s_cap_height: FWORD,

    us_default_char: uint16,
    us_break_char: uint16,
    us_max_context: uint16,
    us_lower_optical_point_size: uint16,
    us_upper_optical_point_size: uint16,

    _mac_style: uint16,
}

#[derive(Clone)]
pub struct OS2Table_v432 {
    version: uint16,

    x_avg_char_width: FWORD,
    us_weight_class: uint16,
    us_width_class: uint16,
    fs_type: uint16,

    y_subscript_x_size: FWORD,
    y_subscript_y_size: FWORD,
    y_subscript_x_offset: FWORD,
    y_subscript_y_offset: FWORD,

    y_superscript_x_size: FWORD,
    y_superscript_y_size: FWORD,
    y_superscript_x_offset: FWORD,
    y_superscript_y_offset: FWORD,

    y_strikeout_size: FWORD,
    y_strikeout_position: FWORD,

    s_family_class: int16,

    panose: [uint8; 10],

    ul_unicode_range1: uint32,
    ul_unicode_range2: uint32,
    ul_unicode_range3: uint32,
    ul_unicode_range4: uint32,

    ach_vend_id: Tag,

    pub fs_selection: uint16,
    us_first_char_index: uint16,
    us_last_char_index: uint16,

    pub s_typo_ascender: FWORD,
    pub s_typo_descender: FWORD,
    pub s_typo_line_gap: FWORD,

    us_win_ascent: UFWORD,
    us_win_descent: UFWORD,

    ul_code_page_range1: uint32,
    ul_code_page_range2: uint32,

    sx_height: FWORD,
    s_cap_height: FWORD,

    us_default_char: uint16,
    us_break_char: uint16,
    us_max_context: uint16,

    _mac_style: uint16,
}

impl Debug for OS2Table_v432 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OS2Table_v432")
            .field("version", &self.version)
            .field("x_avg_char_width", &self.x_avg_char_width)
            .field(
                "us_weight_class",
                &format!(
                    "{} ({})",
                    self.us_weight_class,
                    weight_to_string(self.us_weight_class)
                ),
            )
            .field(
                "us_width_class",
                &format!(
                    "{} ({})",
                    self.us_width_class,
                    width_to_string(self.us_width_class)
                ),
            )
            .field(
                "fs_type",
                &format!("{} ({})", self.fs_type, fstype_to_string(self.fs_type)),
            )
            .field("y_subscript_x_size", &self.y_subscript_x_size)
            .field("y_subscript_y_size", &self.y_subscript_y_size)
            .field("y_subscript_x_offset", &self.y_subscript_x_offset)
            .field("y_subscript_y_offset", &self.y_subscript_y_offset)
            .field("y_superscript_x_size", &self.y_superscript_x_size)
            .field("y_superscript_y_size", &self.y_superscript_y_size)
            .field("y_superscript_x_offset", &self.y_superscript_x_offset)
            .field("y_superscript_y_offset", &self.y_superscript_y_offset)
            .field("y_strikeout_size", &self.y_strikeout_size)
            .field("y_strikeout_position", &self.y_strikeout_position)
            .field("s_family_class", &self.s_family_class)
            .field("panose", &panose_to_string(&self.panose))
            .field("ul_unicode_range1", &self.ul_unicode_range1)
            .field("ul_unicode_range2", &self.ul_unicode_range2)
            .field("ul_unicode_range3", &self.ul_unicode_range3)
            .field("ul_unicode_range4", &self.ul_unicode_range4)
            .field(
                "ach_vend_id",
                &tag_as_str(&self.ach_vend_id).unwrap_or(String::from("????")),
            )
            .field(
                "fs_selection",
                &format!(
                    "{} ({})",
                    self.fs_selection,
                    fs_selection_to_string(self.fs_selection, self._mac_style)
                ),
            )
            .field("us_first_char_index", &self.us_first_char_index)
            .field("us_last_char_index", &self.us_last_char_index)
            .field("s_typo_ascender", &self.s_typo_ascender)
            .field("s_typo_descender", &self.s_typo_descender)
            .field("s_typo_line_gap", &self.s_typo_line_gap)
            .field("us_win_ascent", &self.us_win_ascent)
            .field("us_win_descent", &self.us_win_descent)
            .field("ul_code_page_range1", &self.ul_code_page_range1)
            .field("ul_code_page_range2", &self.ul_code_page_range2)
            .field("sx_height", &self.sx_height)
            .field("s_cap_height", &self.s_cap_height)
            .field("us_default_char", &self.us_default_char)
            .field("us_break_char", &self.us_break_char)
            .field("us_max_context", &self.us_max_context)
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OS2Table_v1 {
    version: uint16,

    x_avg_char_width: FWORD,
    us_weight_class: uint16,
    us_width_class: uint16,
    fs_type: uint16,

    y_subscript_x_size: FWORD,
    y_subscript_y_size: FWORD,
    y_subscript_x_offset: FWORD,
    y_subscript_y_offset: FWORD,

    y_superscript_x_size: FWORD,
    y_superscript_y_size: FWORD,
    y_superscript_x_offset: FWORD,
    y_superscript_y_offset: FWORD,

    y_strikeout_size: FWORD,
    y_strikeout_position: FWORD,

    s_family_class: int16,

    panose: [uint8; 10],

    ul_unicode_range1: uint32,
    ul_unicode_range2: uint32,
    ul_unicode_range3: uint32,
    ul_unicode_range4: uint32,

    ach_vend_id: Tag,

    pub fs_selection: uint16,
    us_first_char_index: uint16,
    us_last_char_index: uint16,

    pub s_typo_ascender: FWORD,
    pub s_typo_descender: FWORD,
    pub s_typo_line_gap: FWORD,

    pub us_win_ascent: UFWORD,
    pub us_win_descent: UFWORD,

    ul_code_page_range1: uint32,
    ul_code_page_range2: uint32,
}

#[derive(Clone, Debug)]
pub enum OS2Table {
    V5(OS2Table_v5),
    V4(OS2Table_v432),
    V3(OS2Table_v432),
    V2(OS2Table_v432),
    V1(OS2Table_v1),

    Interim(uint16),
}

impl TableTrait for OS2Table {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        let version = uint16::from_data(&data[0..2]);
        match version {
            5 => OS2Table::V5(OS2Table_v5 {
                version,

                x_avg_char_width: FWORD::from_data(&data[2..4]),
                us_weight_class: uint16::from_data(&data[4..6]),
                us_width_class: uint16::from_data(&data[6..8]),
                fs_type: uint16::from_data(&data[8..10]),

                y_subscript_x_size: FWORD::from_data(&data[10..12]),
                y_subscript_y_size: FWORD::from_data(&data[12..14]),
                y_subscript_x_offset: FWORD::from_data(&data[14..16]),
                y_subscript_y_offset: FWORD::from_data(&data[16..18]),

                y_superscript_x_size: FWORD::from_data(&data[18..20]),
                y_superscript_y_size: FWORD::from_data(&data[20..22]),
                y_superscript_x_offset: FWORD::from_data(&data[22..24]),
                y_superscript_y_offset: FWORD::from_data(&data[24..26]),

                y_strikeout_size: FWORD::from_data(&data[26..28]),
                y_strikeout_position: FWORD::from_data(&data[28..30]),

                s_family_class: int16::from_data(&data[30..32]),

                panose: data[32..42].try_into().unwrap(),

                ul_unicode_range1: uint32::from_data(&data[42..46]),
                ul_unicode_range2: uint32::from_data(&data[46..50]),
                ul_unicode_range3: uint32::from_data(&data[50..54]),
                ul_unicode_range4: uint32::from_data(&data[54..58]),

                ach_vend_id: data[58..62].try_into().unwrap(),

                fs_selection: uint16::from_data(&data[62..64]),
                us_first_char_index: uint16::from_data(&data[64..66]),
                us_last_char_index: uint16::from_data(&data[66..68]),

                s_typo_ascender: FWORD::from_data(&data[68..70]),
                s_typo_descender: FWORD::from_data(&data[70..72]),
                s_typo_line_gap: FWORD::from_data(&data[72..74]),

                us_win_ascent: UFWORD::from_data(&data[74..76]),
                us_win_descent: UFWORD::from_data(&data[76..78]),

                ul_code_page_range1: uint32::from_data(&data[78..82]),
                ul_code_page_range2: uint32::from_data(&data[82..86]),

                sx_height: FWORD::from_data(&data[86..88]),
                s_cap_height: FWORD::from_data(&data[88..90]),

                us_default_char: uint16::from_data(&data[90..92]),
                us_break_char: uint16::from_data(&data[92..94]),
                us_max_context: uint16::from_data(&data[94..96]),
                us_lower_optical_point_size: uint16::from_data(&data[96..98]),
                us_upper_optical_point_size: uint16::from_data(&data[98..100]),

                _mac_style: if let Some(ParseContext::OS2(mac_style)) = _ctx {
                    mac_style
                } else {
                    0
                },
            }),
            2 | 3 | 4 => {
                let ostable = OS2Table_v432 {
                    version,

                    x_avg_char_width: FWORD::from_data(&data[2..4]),
                    us_weight_class: uint16::from_data(&data[4..6]),
                    us_width_class: uint16::from_data(&data[6..8]),
                    fs_type: uint16::from_data(&data[8..10]),

                    y_subscript_x_size: FWORD::from_data(&data[10..12]),
                    y_subscript_y_size: FWORD::from_data(&data[12..14]),
                    y_subscript_x_offset: FWORD::from_data(&data[14..16]),
                    y_subscript_y_offset: FWORD::from_data(&data[16..18]),

                    y_superscript_x_size: FWORD::from_data(&data[18..20]),
                    y_superscript_y_size: FWORD::from_data(&data[20..22]),
                    y_superscript_x_offset: FWORD::from_data(&data[22..24]),
                    y_superscript_y_offset: FWORD::from_data(&data[24..26]),

                    y_strikeout_size: FWORD::from_data(&data[26..28]),
                    y_strikeout_position: FWORD::from_data(&data[28..30]),

                    s_family_class: int16::from_data(&data[30..32]),

                    panose: data[32..42].try_into().unwrap(),

                    ul_unicode_range1: uint32::from_data(&data[42..46]),
                    ul_unicode_range2: uint32::from_data(&data[46..50]),
                    ul_unicode_range3: uint32::from_data(&data[50..54]),
                    ul_unicode_range4: uint32::from_data(&data[54..58]),

                    ach_vend_id: data[58..62].try_into().unwrap(),

                    fs_selection: uint16::from_data(&data[62..64]),
                    us_first_char_index: uint16::from_data(&data[64..66]),
                    us_last_char_index: uint16::from_data(&data[66..68]),

                    s_typo_ascender: FWORD::from_data(&data[68..70]),
                    s_typo_descender: FWORD::from_data(&data[70..72]),
                    s_typo_line_gap: FWORD::from_data(&data[72..74]),

                    us_win_ascent: UFWORD::from_data(&data[74..76]),
                    us_win_descent: UFWORD::from_data(&data[76..78]),

                    ul_code_page_range1: uint32::from_data(&data[78..82]),
                    ul_code_page_range2: uint32::from_data(&data[82..86]),

                    sx_height: FWORD::from_data(&data[86..88]),
                    s_cap_height: FWORD::from_data(&data[88..90]),

                    us_default_char: uint16::from_data(&data[90..92]),
                    us_break_char: uint16::from_data(&data[92..94]),
                    us_max_context: uint16::from_data(&data[94..96]),

                    _mac_style: if let Some(ParseContext::OS2(mac_style)) = _ctx {
                        mac_style
                    } else {
                        0
                    },
                };

                match version {
                    4 => OS2Table::V4(ostable),
                    3 => OS2Table::V3(ostable),
                    2 => OS2Table::V2(ostable),
                    _ => unreachable!(),
                }
            }
            1 => OS2Table::V1(OS2Table_v1 {
                version,

                x_avg_char_width: FWORD::from_data(&data[2..4]),
                us_weight_class: uint16::from_data(&data[4..6]),
                us_width_class: uint16::from_data(&data[6..8]),
                fs_type: uint16::from_data(&data[8..10]),

                y_subscript_x_size: FWORD::from_data(&data[10..12]),
                y_subscript_y_size: FWORD::from_data(&data[12..14]),
                y_subscript_x_offset: FWORD::from_data(&data[14..16]),
                y_subscript_y_offset: FWORD::from_data(&data[16..18]),

                y_superscript_x_size: FWORD::from_data(&data[18..20]),
                y_superscript_y_size: FWORD::from_data(&data[20..22]),
                y_superscript_x_offset: FWORD::from_data(&data[22..24]),
                y_superscript_y_offset: FWORD::from_data(&data[24..26]),

                y_strikeout_size: FWORD::from_data(&data[26..28]),
                y_strikeout_position: FWORD::from_data(&data[28..30]),

                s_family_class: int16::from_data(&data[30..32]),

                panose: data[32..42].try_into().unwrap(),

                ul_unicode_range1: uint32::from_data(&data[42..46]),
                ul_unicode_range2: uint32::from_data(&data[46..50]),
                ul_unicode_range3: uint32::from_data(&data[50..54]),
                ul_unicode_range4: uint32::from_data(&data[54..58]),

                ach_vend_id: data[58..62].try_into().unwrap(),

                fs_selection: uint16::from_data(&data[62..64]),
                us_first_char_index: uint16::from_data(&data[64..66]),
                us_last_char_index: uint16::from_data(&data[66..68]),

                s_typo_ascender: FWORD::from_data(&data[68..70]),
                s_typo_descender: FWORD::from_data(&data[70..72]),
                s_typo_line_gap: FWORD::from_data(&data[72..74]),

                us_win_ascent: UFWORD::from_data(&data[74..76]),
                us_win_descent: UFWORD::from_data(&data[76..78]),

                ul_code_page_range1: uint32::from_data(&data[78..82]),
                ul_code_page_range2: uint32::from_data(&data[82..86]),
            }),
            _ => {
                panic!("Unsupported OS/2 table version: {}", version);
            }
        }
    }

    fn construct(&mut self, data: &[u8]) {
        if let OS2Table::Interim(mac_style) = self {
            *self = OS2Table::parse(data, Some(ParseContext::OS2(mac_style.clone())));
        } else {
            panic!("OS2Table does not require construction - simply use OS2Table::parse()");
        }
    }
}

impl OS2Table {
    pub fn empty_with_mac(mac_style: uint16) -> OS2Table {
        OS2Table::Interim(mac_style)
    }

    pub fn weight(&self) -> Option<uint16> {
        match self {
            OS2Table::V5(table) => Some(table.us_weight_class),
            OS2Table::V4(table) | OS2Table::V3(table) | OS2Table::V2(table) => {
                Some(table.us_weight_class)
            }
            OS2Table::V1(table) => Some(table.us_weight_class),
            OS2Table::Interim(_) => None,
        }
    }

    pub fn is_italic(&self) -> Option<bool> {
        match self {
            OS2Table::V5(table) => Some(
                (table.fs_selection & FSSelectionFlags::Italic as uint16) != 0
                    && (table._mac_style & MacStyle::Italic) != 0,
            ),
            OS2Table::V4(table) | OS2Table::V3(table) | OS2Table::V2(table) => Some(
                (table.fs_selection & FSSelectionFlags::Italic as uint16) != 0
                    && (table._mac_style & MacStyle::Italic) != 0,
            ),
            OS2Table::V1(table) => {
                Some((table.fs_selection & FSSelectionFlags::Italic as uint16) != 0)
            }
            OS2Table::Interim(_) => None,
        }
    }
}
