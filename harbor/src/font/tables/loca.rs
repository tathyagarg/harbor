#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

/// Requires `head` and `maxp`
#[derive(Clone)]
pub enum LocaTable {
    Short(Vec<uint16>),
    Long(Vec<uint32>),

    /// (index to loc format (head table), number of glyphs (maxp table))
    Interim((int16, uint16)),
}

impl Debug for LocaTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocaTable::Short(offsets) => f
                .debug_struct("LocaTable::Short")
                .field("offsets", &format!("[{} offsets]", offsets.len()))
                .field(
                    "offsets_preview",
                    &offsets.iter().take(4).collect::<Vec<&uint16>>(),
                )
                .finish(),
            LocaTable::Long(offsets) => f
                .debug_struct("LocaTable::Long")
                .field("offsets", &format!("[{} offsets]", offsets.len()))
                .field(
                    "offsets_preview",
                    &offsets.iter().take(4).collect::<Vec<&uint32>>(),
                )
                .finish(),
            _ => panic!("LocaTable::Interim cannot be formatted"),
        }
    }
}

impl TableTrait for LocaTable {
    fn parse(data: &[u8], ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        if let Some(ParseContext::Loca((index_to_loc_format, num_glyphs))) = ctx {
            match index_to_loc_format {
                0 => {
                    // Short format
                    let mut offsets = Vec::with_capacity((num_glyphs + 1) as usize);
                    for i in 0..=(num_glyphs as usize) {
                        let offset = uint16::from_data(&data[i * 2..i * 2 + 2]);
                        offsets.push(offset);
                    }
                    LocaTable::Short(offsets)
                }
                1 => {
                    // Long format
                    let mut offsets = Vec::with_capacity((num_glyphs + 1) as usize);
                    for i in 0..=(num_glyphs as usize) {
                        let offset = uint32::from_data(&data[i * 4..i * 4 + 4]);
                        offsets.push(offset);
                    }
                    LocaTable::Long(offsets)
                }
                _ => panic!("Invalid indexToLocFormat value"),
            }
        } else {
            panic!("LocaTable requires ParseContext::Loca");
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        if let LocaTable::Interim((index_to_loc_format, num_glyphs)) = self {
            *self = LocaTable::parse(
                _data,
                Some(ParseContext::Loca((*index_to_loc_format, *num_glyphs))),
            );
        } else {
            panic!("LocaTable::construct can only be called on Interim variant");
        }
    }
}
