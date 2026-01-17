#![allow(dead_code)]
#![allow(non_camel_case_types)]

/// The goat
use std::fmt::Debug;
use std::ops::BitAnd;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};
use crate::render::text::Segment;

#[repr(u8)]
pub enum SimpleGlyphFlags {
    OnCurvePoint = 0x01,
    XShortVector = 0x02,
    YShortVector = 0x04,
    RepeatFlag = 0x08,
    XIsSameOrPositiveXShortVector = 0x10,
    YIsSameOrPositiveYShortVector = 0x20,
    OverlapSimple = 0x40,
}

impl BitAnd<SimpleGlyphFlags> for uint8 {
    type Output = uint8;

    fn bitand(self, rhs: SimpleGlyphFlags) -> Self::Output {
        self & (rhs as uint8)
    }
}

#[repr(u16)]
pub enum CompositeGlyphFlags {
    Arg1And2AreWords = 0x0001,
    ArgsAreXYValues = 0x0002,
    RoundXYToGrid = 0x0004,
    WeHaveAScale = 0x0008,
    MoreComponents = 0x0020,
    WeHaveAnXAndYScale = 0x0040,
    WeHaveATwoByTwo = 0x0080,
    WeHaveInstructions = 0x0100,
    UseMyMetrics = 0x0200,
    OverlapCompound = 0x0400,
    ScaledComponentOffset = 0x0800,
    UnscaledComponentOffset = 0x1000,
}

impl BitAnd<CompositeGlyphFlags> for uint16 {
    type Output = uint16;

    fn bitand(self, rhs: CompositeGlyphFlags) -> Self::Output {
        self & (rhs as uint16)
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

#[derive(Clone, Debug)]
pub struct Point {
    pub x: int16,
    pub y: int16,

    pub on_curve: bool,
}

impl Point {
    pub fn empty() -> Self {
        Point {
            x: 0,
            y: 0,
            on_curve: false,
        }
    }

    pub fn midpoint(first: &Point, second: &Point) -> Self {
        Point {
            x: ((first.x as i32 + second.x as i32) / 2) as int16,
            y: ((first.y as i32 + second.y as i32) / 2) as int16,
            on_curve: true,
        }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = (other.x - self.x) as f32;
        let dy = (other.y - self.y) as f32;

        (dx * dx + dy * dy).sqrt()
    }

    pub fn distance_to_line(&self, line_start: &Point, line_end: &Point) -> f32 {
        let a = line_end.y as f64 - line_start.y as f64;
        let b = line_start.x as f64 - line_end.x as f64;
        let c = line_end.x as f64 * line_start.y as f64 - line_start.x as f64 * line_end.y as f64;

        let numerator = (a * self.x as f64 + b * self.y as f64 + c).abs();
        let denominator = (a * a + b * b).sqrt();

        if denominator == 0.0 {
            return 0.0;
        }

        (numerator / denominator) as f32
    }

    pub fn transformed(&self, transform: Option<GlyphTransform>) -> Self {
        if let Some(transform) = transform {
            match transform {
                GlyphTransform::Scale(s) => Point {
                    x: ((self.x as f32) * s) as int16,
                    y: ((self.y as f32) * s) as int16,
                    on_curve: self.on_curve,
                },
                GlyphTransform::ScaleXY { x_scale, y_scale } => Point {
                    x: ((self.x as f32) * x_scale) as int16,
                    y: ((self.y as f32) * y_scale) as int16,
                    on_curve: self.on_curve,
                },
                GlyphTransform::Matrix { a, b, c, d } => Point {
                    x: ((self.x as f32) * a + (self.y as f32) * c) as int16,
                    y: ((self.x as f32) * b + (self.y as f32) * d) as int16,
                    on_curve: self.on_curve,
                },
            }
        } else {
            self.clone()
        }
    }

    pub fn translate(&self, dx: int16, dy: int16) -> Self {
        Point {
            x: self.x + dx,
            y: self.y + dy,
            on_curve: self.on_curve,
        }
    }

    pub fn vertex_coords(&self, origin: (f32, f32), scale: f32) -> (f32, f32) {
        let scaled_x = origin.0 + self.x as f32 * scale;
        let scaled_y = origin.1 - self.y as f32 * scale;

        (scaled_x, scaled_y)
    }

    pub fn vertex_position(&self, origin: (f32, f32), scale: f32) -> [f32; 3] {
        let (scaled_x, scaled_y) = self.vertex_coords(origin, scale);

        [scaled_x, scaled_y, 0.0]
    }
}

#[derive(Clone, Debug)]
pub struct Contour {
    pub points: Vec<Point>,

    pub length: usize,
}

impl Contour {
    pub fn to_segments(&self) -> Vec<Segment> {
        let mut segments = Vec::new();

        let num_points = self.points.len();

        for i in 0..num_points {
            let current_point = &self.points[i];
            let next_point = &self.points[(i + 1) % num_points];

            if current_point.on_curve && next_point.on_curve {
                // Line segment
                segments.push(Segment::Line(current_point.clone(), next_point.clone()));
            } else if current_point.on_curve && !next_point.on_curve {
                // Quadratic Bezier segment
                let after_next_point = &self.points[(i + 2) % num_points];

                let control_point = next_point.clone();
                let end_point = if after_next_point.on_curve {
                    after_next_point.clone()
                } else {
                    Point::midpoint(next_point, after_next_point)
                };

                segments.push(Segment::Quadratic(
                    current_point.clone(),
                    control_point,
                    end_point,
                ));
            }
        }

        segments
    }
}

#[derive(Clone)]
pub struct SimpleGlyphData {
    pub end_pts_of_contours: Vec<uint16>,
    pub instruction_length: uint16,
    pub instructions: Vec<uint8>,

    pub contours: Vec<Contour>,
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
            .field("contours_length", &self.contours.len())
            .field(
                "contours_preview",
                &self.contours.iter().take(2).collect::<Vec<&Contour>>(),
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum GlyphTransform {
    Scale(f32),
    ScaleXY { x_scale: f32, y_scale: f32 },
    Matrix { a: f32, b: f32, c: f32, d: f32 },
}

#[derive(Clone, Default)]
pub struct GlyphComponent {
    pub flags: uint16,
    pub glyph_index: uint16,

    pub arg1: int16,
    pub arg2: int16,

    pub transform: Option<GlyphTransform>,
}

impl Debug for GlyphComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphComponent")
            .field("flags", &format!("{:016b}", self.flags))
            .field("glyph_index", &self.glyph_index)
            .field("arg1", &self.arg1)
            .field("arg2", &self.arg2)
            .field("transform", &self.transform)
            .finish()
    }
}

#[derive(Clone)]
pub struct CompositeGlyphData {
    pub components: Vec<GlyphComponent>,

    pub instructions: Option<Vec<uint8>>,
}

impl Debug for CompositeGlyphData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeGlyphData")
            .field("components", &self.components)
            .field(
                "instructions_preview",
                &self
                    .instructions
                    .as_ref()
                    .map(|ins| ins.iter().take(10).collect::<Vec<&uint8>>()),
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum GlyphDataType {
    Simple(SimpleGlyphData),
    Composite(CompositeGlyphData),
}

#[derive(Clone, Debug)]
pub struct GlyphData {
    pub header: GlyphHeader,

    pub data: GlyphDataType,
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
                // &self.glyphs,
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
                        contours: vec![],
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

            let mut offset = 10;
            if number_of_contours >= 0 {
                // Simple glyph

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

                let mut contours = Vec::<Contour>::with_capacity(number_of_contours as usize);
                let mut prev_countour_end = 0;

                let mut total_length = 0;

                for contour_index in 0..number_of_contours as usize {
                    let contour_length = end_pts_of_contours[contour_index] - prev_countour_end + 1;
                    prev_countour_end = end_pts_of_contours[contour_index] + 1;

                    total_length += contour_length as usize;

                    let mut points = Vec::new();

                    for _ in 0..contour_length {
                        points.push(Point::empty());
                    }

                    contours.push(Contour {
                        points,
                        length: contour_length as usize,
                    });
                }

                let mut flags = Vec::new();

                while flags.len() < total_length {
                    let flag = glyph_data[offset];
                    offset += 1;

                    flags.push(flag);

                    if flag & SimpleGlyphFlags::RepeatFlag != 0 {
                        let repeat_count = glyph_data[offset];
                        offset += 1;

                        for _ in 0..repeat_count {
                            flags.push(flag);
                        }
                    }
                }

                assert!(flags.len() == total_length);

                let mut curr_point_index = 0;

                for contour in &mut contours {
                    for i in 0..contour.length {
                        contour.points[i].on_curve =
                            (flags[curr_point_index] & SimpleGlyphFlags::OnCurvePoint) != 0;

                        curr_point_index += 1;
                    }
                }

                let mut prev_x = 0;
                let mut curr_flag_index = 0;

                for contour in &mut contours {
                    for i in 0..contour.length {
                        let flag = flags[curr_flag_index];
                        curr_flag_index += 1;

                        let dx = if flag & SimpleGlyphFlags::XShortVector != 0 {
                            let x_byte = glyph_data[offset];
                            offset += 1;

                            let x_val =
                                if flag & SimpleGlyphFlags::XIsSameOrPositiveXShortVector != 0 {
                                    x_byte as int16
                                } else {
                                    -(x_byte as int16)
                                };

                            x_val
                        } else {
                            if flag & SimpleGlyphFlags::XIsSameOrPositiveXShortVector != 0 {
                                0
                            } else {
                                let dx = int16::from_data(&glyph_data[offset..offset + 2]);
                                offset += 2;

                                dx
                            }
                        };

                        contour.points[i].x = prev_x + dx;
                        prev_x = contour.points[i].x;
                    }
                }

                let mut prev_y = 0;
                curr_flag_index = 0;

                for contour in &mut contours {
                    for i in 0..contour.length {
                        let flag = flags[curr_flag_index];
                        curr_flag_index += 1;

                        let dy = if flag & SimpleGlyphFlags::YShortVector != 0 {
                            let y_byte = glyph_data[offset];
                            offset += 1;

                            let y_val =
                                if flag & SimpleGlyphFlags::YIsSameOrPositiveYShortVector != 0 {
                                    y_byte as int16
                                } else {
                                    -(y_byte as int16)
                                };

                            y_val
                        } else {
                            if flag & SimpleGlyphFlags::YIsSameOrPositiveYShortVector != 0 {
                                0
                            } else {
                                let dy = int16::from_data(&glyph_data[offset..offset + 2]);
                                offset += 2;

                                dy
                            }
                        };

                        contour.points[i].y = prev_y + dy;
                        prev_y = contour.points[i].y;
                    }
                }

                self.glyphs.push(GlyphData {
                    header,
                    data: GlyphDataType::Simple(SimpleGlyphData {
                        end_pts_of_contours,
                        instruction_length,
                        instructions,
                        contours,
                    }),
                })
            } else {
                // Composite glyph
                let mut components = Vec::<GlyphComponent>::new();

                let mut we_have_instructions = false;

                loop {
                    let mut component = GlyphComponent::default();

                    let flags = uint16::from_data(&glyph_data[offset..offset + 2]);
                    offset += 2;
                    component.flags = flags;

                    we_have_instructions = we_have_instructions
                        || (flags & CompositeGlyphFlags::WeHaveInstructions != 0);

                    let _glyph_index = uint16::from_data(&glyph_data[offset..offset + 2]);
                    offset += 2;
                    component.glyph_index = _glyph_index;

                    let arg1_and_2_are_words = flags & CompositeGlyphFlags::Arg1And2AreWords != 0;

                    let (arg1, arg2) = if arg1_and_2_are_words {
                        let result = (
                            int16::from_data(&glyph_data[offset..offset + 2]),
                            int16::from_data(&glyph_data[offset + 2..offset + 4]),
                        );

                        offset += 4;

                        result
                    } else {
                        let result = (glyph_data[offset] as int16, glyph_data[offset + 1] as int16);

                        offset += 2;

                        result
                    };

                    component.arg1 = arg1;
                    component.arg2 = arg2;

                    component.transform = if flags & CompositeGlyphFlags::WeHaveAScale != 0 {
                        let scale =
                            f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset..offset + 2]));

                        offset += 2;

                        Some(GlyphTransform::Scale(scale))
                    } else if flags & CompositeGlyphFlags::WeHaveAnXAndYScale != 0 {
                        let x_scale =
                            f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset..offset + 2]));
                        let y_scale =
                            f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset + 2..offset + 4]));
                        offset += 4;

                        Some(GlyphTransform::ScaleXY { x_scale, y_scale })
                    } else if flags & CompositeGlyphFlags::WeHaveATwoByTwo != 0 {
                        let a = f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset..offset + 2]));
                        let b =
                            f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset + 2..offset + 4]));
                        let c =
                            f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset + 4..offset + 6]));
                        let d =
                            f2dot14_to_f32(F2DOT14::from_data(&glyph_data[offset + 6..offset + 8]));
                        offset += 8;

                        Some(GlyphTransform::Matrix { a, b, c, d })
                    } else {
                        None
                    };

                    components.push(component);

                    if flags & CompositeGlyphFlags::MoreComponents == 0 {
                        break;
                    }
                }

                let instructions = if we_have_instructions {
                    let instruction_length = uint16::from_data(&glyph_data[offset..offset + 2]);
                    let instructions =
                        &glyph_data[offset + 2..offset + 2 + (instruction_length as usize)];

                    Some(instructions)
                } else {
                    None
                };

                self.glyphs.push(GlyphData {
                    header,
                    data: GlyphDataType::Composite(CompositeGlyphData {
                        components,
                        instructions: instructions.map(|ins| ins.to_vec()),
                    }),
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
