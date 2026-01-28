use std::fmt::Debug;

use crate::{
    css::colors::UsedColor,
    font::tables::glyf::{GlyphTransform, Point},
};

#[derive(Clone)]
pub enum Segment {
    Line(Point, Point),
    Quadratic(Point, Point, Point),
}

impl Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Line(p0, p1) => f
                .debug_struct("Line")
                .field("p0", p0)
                .field("p1", p1)
                .finish(),
            Segment::Quadratic(p0, c, p2) => f
                .debug_struct("Quadratic")
                .field("p0", p0)
                .field("c", c)
                .field("p2", p2)
                .finish(),
        }
    }
}

impl Segment {
    pub fn flatten(&self, out: &mut Vec<Point>, precision: f32) {
        match self {
            Segment::Line(p0, p1) => {
                out.push(p0.clone());
                out.push(p1.clone());
            }
            Segment::Quadratic(p0, c, p2) => {
                if c.distance_to_line(p0, p2) < 5.0 {
                    out.push(p0.clone());
                    out.push(p2.clone());
                } else {
                    let mid1 = Point::midpoint(p0, c);
                    let mid2 = Point::midpoint(c, p2);
                    let mid = Point::midpoint(&mid1, &mid2);

                    Segment::Quadratic(p0.clone(), mid1, mid.clone()).flatten(out, precision);
                    Segment::Quadratic(mid, mid2, p2.clone()).flatten(out, precision);
                }
            }
        }
    }

    pub fn transformed(&self, transform: Option<GlyphTransform>) -> Segment {
        match self {
            Segment::Line(p0, p1) => Segment::Line(
                p0.transformed(transform.clone()),
                p1.transformed(transform.clone()),
            ),
            Segment::Quadratic(p0, c, p2) => Segment::Quadratic(
                p0.transformed(transform.clone()),
                c.transformed(transform.clone()),
                p2.transformed(transform.clone()),
            ),
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        *self = match self {
            Segment::Line(p0, p1) => Segment::Line(p0.translate(dx, dy), p1.translate(dx, dy)),
            Segment::Quadratic(p0, c, p2) => Segment::Quadratic(
                p0.translate(dx, dy),
                c.translate(dx, dy),
                p2.translate(dx, dy),
            ),
        }
    }
}

#[derive(Clone)]
pub struct GlyphMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub advance_width: f32,

    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct GlyphInstance {
    pub offset: [f32; 2],
    pub color: UsedColor,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlyphVertex {
    pub position: [f32; 2],
}
