#![allow(dead_code)]
#![allow(non_camel_case_types)]

/// The goat
use std::fmt::Debug;
use std::ops::BitAnd;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

#[repr(u8)]
pub enum GlyphFlags {
    OnCurvePoint = 0x01,
    XShortVector = 0x02,
    YShortVector = 0x04,
    RepeatFlag = 0x08,
    XIsSameOrPositiveXShortVector = 0x10,
    YIsSameOrPositiveYShortVector = 0x20,
    OverlapSimple = 0x40,
}

impl BitAnd<GlyphFlags> for uint8 {
    type Output = uint8;

    fn bitand(self, rhs: GlyphFlags) -> Self::Output {
        self & (rhs as uint8)
    }
}

#[derive(Clone, Debug)]
pub struct GlyphHeader {
    /// If the number of contours is greater than or equal to zero, this is a simple glyph. If
    /// negative, this is a composite glyph - the value -1 should be used for composite glyphs.
    pub number_of_contours: int16,

    /// Minimum x for coordinate data.
    pub x_min: int16,

    /// Minimum y for coordinate data.
    pub y_min: int16,

    /// Maximum x for coordinate data.
    pub x_max: int16,

    /// Maximum y for coordinate data.
    pub y_max: int16,
}

#[derive(Clone)]
pub struct SimpleGlyphData {
    end_pts_of_contours: Vec<uint16>,
    instruction_length: uint16,
    instructions: Vec<uint8>,

    flags: Vec<uint8>,

    x_coordinates: Vec<int16>,
    y_coordinates: Vec<int16>,
}

impl Debug for SimpleGlyphData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleGlyphData")
            .field("end_pts_of_contours", &self.end_pts_of_contours)
            .field("instruction_length", &self.instruction_length)
            .field(
                "instructions_preview",
                &self.instructions.iter().take(10).collect::<Vec<&uint8>>(),
            )
            .field(
                "flags_preview",
                &self
                    .flags
                    .iter()
                    .take(10)
                    .map(|b| format!("{:08b}", b))
                    .collect::<Vec<String>>(),
            )
            .field("x_coordinates_length", &self.x_coordinates.len())
            .field(
                "x_coordinates_preview",
                &self.x_coordinates.iter().take(10).collect::<Vec<&int16>>(),
            )
            .field("y_coordinates_length", &self.y_coordinates.len())
            .field(
                "y_coordinates_preview",
                &self.y_coordinates.iter().take(10).collect::<Vec<&int16>>(),
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum GlyphDataType {
    Simple(SimpleGlyphData),
    Composite,
}

#[derive(Clone, Debug)]
pub struct GlyphData {
    header: GlyphHeader,

    data: GlyphDataType,
}

#[derive(Clone, Default)]
pub struct GlyfTable {
    pub glyphs: Vec<GlyphData>,

    _loca_offsets: Vec<uint32>,
}

impl Debug for GlyfTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyfTable")
            .field("glyphs_count", &self.glyphs.len())
            .field(
                "glyphs_preview",
                &self.glyphs.iter().take(5).collect::<Vec<&GlyphData>>(),
            )
            .finish()
    }
}

impl TableTrait for GlyfTable {
    fn parse(_data: &[u8], _ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        panic!(
            "Cannot parse GlyfTable without loca offsets. Use GlyfTable::default and then \
             GlyfTable::with_locas to provide loca offsets."
        );
    }

    fn construct(&mut self, data: &[u8]) {
        for i in 0..(self._loca_offsets.len() - 1) {
            let start = self._loca_offsets[i] as usize;
            let end = self._loca_offsets[i + 1] as usize;

            if start == end {
                // Empty glyph
                self.glyphs.push(GlyphData {
                    header: GlyphHeader {
                        number_of_contours: 0,
                        x_min: 0,
                        y_min: 0,
                        x_max: 0,
                        y_max: 0,
                    },
                    data: GlyphDataType::Simple(SimpleGlyphData {
                        end_pts_of_contours: vec![],
                        instruction_length: 0,
                        instructions: vec![],
                        flags: vec![],
                        x_coordinates: vec![],
                        y_coordinates: vec![],
                    }),
                });
                continue;
            }

            let glyph_data = &data[start..end];
            let number_of_contours = int16::from_data(&glyph_data[0..2]);
            let x_min = int16::from_data(&glyph_data[2..4]);
            let y_min = int16::from_data(&glyph_data[4..6]);
            let x_max = int16::from_data(&glyph_data[6..8]);
            let y_max = int16::from_data(&glyph_data[8..10]);

            let header = GlyphHeader {
                number_of_contours,
                x_min,
                y_min,
                x_max,
                y_max,
            };

            if number_of_contours >= 0 {
                // Simple glyph
                let mut offset = 10;

                let mut end_pts_of_contours =
                    Vec::<uint16>::with_capacity(number_of_contours as usize);
                for _ in 0..number_of_contours {
                    let end_pt = uint16::from_data(&glyph_data[offset..offset + 2]);
                    end_pts_of_contours.push(end_pt);
                    offset += 2;
                }

                let instruction_length = uint16::from_data(&glyph_data[offset..offset + 2]);
                offset += 2;

                let instructions =
                    glyph_data[offset..offset + (instruction_length as usize)].to_vec();
                offset += instruction_length as usize;

                let mut flags = Vec::<uint8>::new();

                while flags.len()
                    < (end_pts_of_contours
                        .last()
                        .map(|v| *v as usize + 1)
                        .unwrap_or(0))
                {
                    let flag = glyph_data[offset];
                    offset += 1;
                    flags.push(flag);

                    if flag & 0x08 != 0 {
                        let repeat_count = glyph_data[offset];
                        offset += 1;
                        for _ in 0..repeat_count {
                            flags.push(flag);
                        }
                    }
                }

                let mut x_coords = Vec::<int16>::new();

                for &flag in &flags {
                    if flag & GlyphFlags::XShortVector != 0 {
                        let x_byte = glyph_data[offset];
                        offset += 1;
                        let x_value = if flag & GlyphFlags::XIsSameOrPositiveXShortVector != 0 {
                            x_byte as int16
                        } else {
                            -(x_byte as int16)
                        };
                        x_coords.push(x_value);
                    } else if flag & GlyphFlags::XIsSameOrPositiveXShortVector == 0 {
                        let x_value = int16::from_data(&glyph_data[offset..offset + 2]);
                        offset += 2;
                        x_coords.push(x_value);
                    } else {
                        x_coords.push(0);
                    }
                }

                let mut y_coords = Vec::<int16>::new();

                for &flag in &flags {
                    if flag & GlyphFlags::YShortVector != 0 {
                        let y_byte = glyph_data[offset];
                        offset += 1;
                        let y_value = if flag & GlyphFlags::YIsSameOrPositiveYShortVector != 0 {
                            y_byte as int16
                        } else {
                            -(y_byte as int16)
                        };
                        y_coords.push(y_value);
                    } else if flag & GlyphFlags::YIsSameOrPositiveYShortVector == 0 {
                        let y_value = int16::from_data(&glyph_data[offset..offset + 2]);
                        offset += 2;
                        y_coords.push(y_value);
                    } else {
                        y_coords.push(0);
                    }
                }

                self.glyphs.push(GlyphData {
                    header,
                    data: GlyphDataType::Simple(SimpleGlyphData {
                        end_pts_of_contours,
                        instruction_length,
                        instructions,
                        flags,
                        x_coordinates: x_coords,
                        y_coordinates: y_coords,
                    }),
                });
            } else {
                // Composite glyph
                self.glyphs.push(GlyphData {
                    header,
                    data: GlyphDataType::Composite,
                });
            }
        }
    }
}

impl GlyfTable {
    pub fn with_locas(mut self, loca_offsets: Vec<uint32>) -> Self {
        self._loca_offsets = loca_offsets;
        self
    }
}
