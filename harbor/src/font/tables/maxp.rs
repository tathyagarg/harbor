#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

#[derive(Clone, Debug)]
pub struct MaxPTable_v0_5 {
    /// Version16Dot16
    /// Fixed version number of the 'maxp' table — set to 0x00005000 for version 0.5.

    /// The number of glyphs in the font.
    pub num_glyphs: uint16,
}

#[derive(Clone, Debug)]
pub struct MaxPTable_v1_0 {
    /// Version16Dot16
    /// Fixed version number of the 'maxp' table — set to 0x00010000 for version 1.0.

    /// The number of glyphs in the font.
    pub num_glyphs: uint16,

    /// Maximum points in a non-composite glyph.
    pub max_points: uint16,

    /// Maximum contours in a non-composite glyph.
    pub max_contours: uint16,

    /// Maximum points in a composite glyph.
    pub max_composite_points: uint16,

    /// Maximum contours in a composite glyph.
    pub max_composite_contours: uint16,

    /// 1 if instructions do not use the twilight zone (Z0), or 2 if instructions do use Z0; should
    /// be set to 2 in most cases.
    pub max_zones: uint16,

    /// Maximum points used in Z0.
    pub max_twilight_points: uint16,

    /// Number of Storage Area locations.
    pub max_storage: uint16,

    /// Number of FDEFs, equal to the highest function number + 1.
    pub max_function_defs: uint16,

    /// Number of IDEFs.
    pub max_instruction_defs: uint16,

    /// Maximum stack depth across Font Program ('fpgm' table), CVT Program ('prep' table) and all
    /// glyph instructions (in the 'glyf' table).
    pub max_stack_elements: uint16,

    /// Maximum byte count for glyph instructions.
    pub max_size_of_instructions: uint16,

    /// Maximum number of components referenced at “top level” for any composite glyph.
    pub max_component_elements: uint16,

    /// Maximum levels of recursion; 1 for simple components.
    pub max_component_depth: uint16,
}

#[derive(Clone, Debug)]
pub enum MaxPTable {
    V0_5(MaxPTable_v0_5),
    V1_0(MaxPTable_v1_0),
}

impl TableTrait for MaxPTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        let version = uint32::from_data(&data[0..4]);
        match version {
            0x00005000 => MaxPTable::V0_5(MaxPTable_v0_5 {
                num_glyphs: uint16::from_data(&data[4..6]),
            }),
            0x00010000 => MaxPTable::V1_0(MaxPTable_v1_0 {
                num_glyphs: uint16::from_data(&data[4..6]),
                max_points: uint16::from_data(&data[6..8]),
                max_contours: uint16::from_data(&data[8..10]),
                max_composite_points: uint16::from_data(&data[10..12]),
                max_composite_contours: uint16::from_data(&data[12..14]),
                max_zones: uint16::from_data(&data[14..16]),
                max_twilight_points: uint16::from_data(&data[16..18]),
                max_storage: uint16::from_data(&data[18..20]),
                max_function_defs: uint16::from_data(&data[20..22]),
                max_instruction_defs: uint16::from_data(&data[22..24]),
                max_stack_elements: uint16::from_data(&data[24..26]),
                max_size_of_instructions: uint16::from_data(&data[26..28]),
                max_component_elements: uint16::from_data(&data[28..30]),
                max_component_depth: uint16::from_data(&data[30..32]),
            }),
            _ => panic!("Unsupported 'maxp' table version: 0x{:08X}", version),
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("MaxPTable does not require construction - simply use MaxPTable::parse()");
    }
}
