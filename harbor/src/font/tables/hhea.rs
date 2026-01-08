#![allow(dead_code)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

#[derive(Clone, Default, Debug)]
pub struct HHeaTable {
    /// Major version number of the horizontal header table — set to 1.
    pub major_version: uint16,

    /// Minor version number of the horizontal header table — set to 0.
    pub minor_version: uint16,

    /// Typographic ascent
    pub ascender: FWORD,

    /// Typographic descent
    pub descender: FWORD,

    /// Typographic line gap
    pub line_gap: FWORD,

    /// Maximum advance width value in 'hmtx' table
    pub advance_width_max: UFWORD,

    /// Minimum left sidebearing value in 'hmtx' table for glyphs with contours
    pub min_left_side_bearing: FWORD,

    /// Minimum right sidebearing value; calculated as min(aw - (lsb + xMax - xMin)) for glyphs
    /// with contours
    pub min_right_side_bearing: FWORD,

    /// Max(lsb + (xMax - xMin)).
    pub x_max_extent: FWORD,

    /// Used to calculate the slope of the cursor (rise/run); 1 for vertical.
    pub caret_slope_rise: int16,

    /// 0 for vertical.
    pub caret_slope_run: int16,

    /// The amount by which a slanted highlight on a glyph needs to be shifted to produce the best appearance. Set to 0 for non-slanted fonts
    pub caret_offset: int16,

    /// Reserved; set to 0: int16 * 4

    /// 0 for current format.
    /// pub metric_data_format: int16,

    /// Number of hMetric entries in 'hmtx' table
    pub number_of_h_metrics: uint16,
}

impl TableTrait for HHeaTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        HHeaTable {
            major_version: uint16::from_data(&data[0..2]),
            minor_version: uint16::from_data(&data[2..4]),
            ascender: FWORD::from_data(&data[4..6]),
            descender: FWORD::from_data(&data[6..8]),
            line_gap: FWORD::from_data(&data[8..10]),
            advance_width_max: UFWORD::from_data(&data[10..12]),
            min_left_side_bearing: FWORD::from_data(&data[12..14]),
            min_right_side_bearing: FWORD::from_data(&data[14..16]),
            x_max_extent: FWORD::from_data(&data[16..18]),
            caret_slope_rise: int16::from_data(&data[18..20]),
            caret_slope_run: int16::from_data(&data[20..22]),
            caret_offset: int16::from_data(&data[22..24]),
            number_of_h_metrics: uint16::from_data(&data[34..36]),
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("HHeaTable does not require construction - simply use HHeaTable::parse()");
    }
}
