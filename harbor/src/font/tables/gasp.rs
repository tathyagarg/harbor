#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;
use std::ops::BitAnd;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

#[repr(u16)]
pub enum GASPBehavior {
    /// Use gridfitting
    Gridfit = 0x0001,

    /// Use grayscale rendering
    DoGray = 0x0002,

    /// Use gridfitting with ClearType symmetric smoothing
    /// Only supported in version 1
    SymmetricSmoothing = 0x0004,

    /// Use smoothing along multiple axes with ClearType®
    /// Only supported in version 1
    SymmetricGridfit = 0x0008,
}

impl BitAnd<GASPBehavior> for uint16 {
    type Output = uint16;

    fn bitand(self, rhs: GASPBehavior) -> Self::Output {
        self & (rhs as uint16)
    }
}

fn gasp_behavior_to_string(behavior: u16) -> String {
    let mut flags = Vec::new();

    if behavior & GASPBehavior::Gridfit != 0 {
        flags.push("Gridfit");
    }
    if behavior & GASPBehavior::DoGray != 0 {
        flags.push("DoGray");
    }
    if behavior & GASPBehavior::SymmetricSmoothing != 0 {
        flags.push("SymmetricSmoothing");
    }
    if behavior & GASPBehavior::SymmetricGridfit != 0 {
        flags.push("SymmetricGridfit");
    }

    flags.join(" | ")
}

#[derive(Clone)]
pub struct GASPRange {
    /// Upper limit of range, in PPEM
    pub range_max_ppem: uint16,

    /// Flags describing desired rasterizer behavior.
    pub range_gasp_behavior: uint16,
}

impl Debug for GASPRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GASPRange")
            .field("range_max_ppem", &self.range_max_ppem)
            .field(
                "range_gasp_behavior",
                &gasp_behavior_to_string(self.range_gasp_behavior),
            )
            .field("raw_range_gasp_behavior", &self.range_gasp_behavior)
            .finish()
    }
}

#[derive(Clone)]
pub struct GASPTable {
    /// Version number (0 or 1—set to 1 in new fonts)
    pub version: uint16,

    /// Number of records to follow
    pub num_ranges: uint16,

    /// Sorted by ppem
    pub ranges: Vec<GASPRange>,
}

impl Debug for GASPTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GASPTable")
            .field("version", &self.version)
            .field("num_ranges", &self.num_ranges)
            .field(
                "ranges_preview",
                &self.ranges.iter().take(5).collect::<Vec<&GASPRange>>(),
            )
            .finish()
    }
}

impl TableTrait for GASPTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        let version = uint16::from_data(&data[0..2]);
        let num_ranges = uint16::from_data(&data[2..4]);

        let mut ranges = Vec::with_capacity(num_ranges as usize);
        for i in 0..num_ranges {
            let offset = 4 + (i as usize) * 4;

            let range_max_ppem = uint16::from_data(&data[offset..offset + 2]);
            let range_gasp_behavior = uint16::from_data(&data[offset + 2..offset + 4]);

            ranges.push(GASPRange {
                range_max_ppem,
                range_gasp_behavior,
            });
        }

        GASPTable {
            version,
            num_ranges,
            ranges,
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("GASPTable does not require construction - simply use GASPTable::parse()");
    }
}
