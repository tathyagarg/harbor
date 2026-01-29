use std::{collections::HashMap, fmt::Debug};

use wgpu::{
    Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    css::colors::UsedColor,
    font::{
        otf_dtypes::GLYPH_ID,
        tables::glyf::{GlyphTransform, Point},
        ttf::TableDirectory,
    },
};

#[derive(Clone)]
pub struct TextRenderer {
    pub _associated_weight: u16,
    pub _associated_italic: bool,

    pub font: TableDirectory,

    /// Key: (glyph_id, font size)
    pub glyph_cache: HashMap<(GLYPH_ID, u32), GlyphMesh>,
}

impl TextRenderer {
    pub fn get_from_char(
        &mut self,
        ch: char,
        font_size: u32,
        device: &Device,
        queue: &wgpu::Queue,
    ) -> Option<GlyphMesh> {
        let glyph_id = self.font.cmap_lookup(ch as u32);

        if let Some(gid) = glyph_id {
            if let Some(glyph) = self.glyph_cache.get(&(gid, font_size)) {
                return Some(glyph.clone());
            } else {
                let mut points: Vec<Point> = Vec::new();
                self.font.make_glyph_points(gid, 5.0, &mut points);

                if points.len() == 0 {
                    return None;
                }

                let scale = font_size as f32 / self.font.units_per_em() as f32;

                let glyph_verts = points
                    .iter()
                    .map(|p| GlyphVertex {
                        position: [p.x * scale, p.y * scale],
                    })
                    .collect::<Vec<GlyphVertex>>();

                let mut contours = Vec::new();

                self.font
                    .make_glyph_points_contours(gid, 5.0, &mut contours);

                let filled_verts = build_filled_glyph_mesh(contours)
                    .iter()
                    .map(|v| GlyphVertex {
                        position: [v.position[0] * scale, v.position[1] * scale],
                    })
                    .collect::<Vec<GlyphVertex>>();

                println!("Filled verts count: {}", filled_verts.len());

                let glyph_mesh = GlyphMesh {
                    outline_vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Glyph Outline Vertex Buffer"),
                        contents: bytemuck::cast_slice(&glyph_verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                    outline_vertex_count: glyph_verts.len() as u32,
                    fill_vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Glyph Fill Vertex Buffer"),
                        contents: bytemuck::cast_slice(&filled_verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                    fill_vertex_count: filled_verts.len() as u32,
                    advance_width: self.font.advance_width(gid).unwrap_or_else(|| {
                        self.font
                            .advance_width(self.font.last_glyph_index().unwrap())
                            .unwrap_or(0)
                    }) as f32
                        * scale,
                    instance_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Glyph Instance Buffer"),
                        size: 10_000 * std::mem::size_of::<GlyphInstance>() as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    instance_count: 0,
                };

                self.glyph_cache
                    .insert((gid, font_size), glyph_mesh.clone());

                Some(glyph_mesh)
            }
        } else {
            None
        }
    }
}

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

    pub fn flatten_no_dup(&self, out: &mut Vec<Point>, precision: f32) {
        match self {
            Segment::Line(p0, p1) => {
                if out.len() == 0 {
                    out.push(p0.clone());
                }
                out.push(p1.clone());
            }
            Segment::Quadratic(p0, c, p2) => {
                if c.distance_to_line(p0, p2) < precision {
                    if out.len() == 0 {
                        out.push(p0.clone());
                    }
                    out.push(p2.clone());
                } else {
                    let mid1 = Point::midpoint(p0, c);
                    let mid2 = Point::midpoint(c, p2);
                    let mid = Point::midpoint(&mid1, &mid2);

                    Segment::Quadratic(p0.clone(), mid1, mid.clone())
                        .flatten_no_dup(out, precision);
                    Segment::Quadratic(mid, mid2, p2.clone()).flatten_no_dup(out, precision);
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
    pub outline_vertex_buffer: wgpu::Buffer,
    pub outline_vertex_count: u32,

    pub fill_vertex_buffer: wgpu::Buffer,
    pub fill_vertex_count: u32,

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
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct GlyphVertex {
    pub position: [f32; 2],
}

impl GlyphVertex {
    #[inline]
    pub fn cross(a: &GlyphVertex, b: &GlyphVertex, c: &GlyphVertex) -> f32 {
        (b.position[0] - a.position[0]) * (c.position[1] - a.position[1])
            - (b.position[1] - a.position[1]) * (c.position[0] - a.position[0])
    }
}

pub fn point_in_triangle(
    p: &GlyphVertex,
    a: &GlyphVertex,
    b: &GlyphVertex,
    c: &GlyphVertex,
) -> bool {
    let abp = GlyphVertex::cross(a, b, p);
    let bcp = GlyphVertex::cross(b, c, p);
    let cap = GlyphVertex::cross(c, a, p);

    (abp >= 0.0 && bcp >= 0.0 && cap >= 0.0) || (abp <= 0.0 && bcp <= 0.0 && cap <= 0.0)
}

/// a: prev
/// b: current
/// c: next
pub fn is_convex(a: &GlyphVertex, b: &GlyphVertex, c: &GlyphVertex) -> bool {
    GlyphVertex::cross(a, b, c) < 0.0
}

pub fn is_ear(
    i_prev: usize,
    i_curr: usize,
    i_next: usize,
    indices: &[usize],
    vertices: &[GlyphVertex],
) -> bool {
    let a = &vertices[i_prev];
    let b = &vertices[i_curr];
    let c = &vertices[i_next];

    println!("Checking ear: a={:?}, b={:?}, c={:?}", a, b, c);

    if !is_convex(a, b, c) {
        println!("Denied: convex");
        return false;
    }

    for &i in indices {
        if i != i_prev && i != i_curr && i != i_next {
            let p = &vertices[i];
            if point_in_triangle(p, a, b, c) {
                println!("Denied: point in triangle {:?}", p);
                return false;
            }
        }
    }

    println!("Accepted");
    true
}

pub fn triangulate_polygon(vertices: &[GlyphVertex]) -> Vec<[GlyphVertex; 3]> {
    let mut indices: Vec<usize> = (0..vertices.len()).collect();
    let mut triangles: Vec<[GlyphVertex; 3]> = Vec::new();

    while indices.len() >= 3 {
        let mut ear_found = false;

        for i in 0..indices.len() {
            let i_prev = indices[(i + indices.len() - 1) % indices.len()];
            let i_curr = indices[i];
            let i_next = indices[(i + 1) % indices.len()];

            if is_ear(i_prev, i_curr, i_next, &indices, vertices) {
                triangles.push([
                    vertices[i_prev].clone(),
                    vertices[i_curr].clone(),
                    vertices[i_next].clone(),
                ]);

                indices.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            // Polygon is not simple or cannot be triangulated
            break;
        }
    }

    println!("Triangles: {:#?}", triangles);

    triangles
}

pub fn build_filled_glyph_mesh(contours: Vec<Vec<Point>>) -> Vec<GlyphVertex> {
    let mut vertices: Vec<GlyphVertex> = Vec::new();

    for contour in contours {
        let contour_vertices: Vec<GlyphVertex> = contour
            .iter()
            .map(|p| GlyphVertex {
                position: [p.x, p.y],
            })
            .collect::<Vec<_>>();

        // .filter_map(|(i, p)| {
        //     if i % 2 == 0 {
        //         Some(GlyphVertex {
        //             position: [p.x, p.y],
        //         })
        //     } else {
        //         None
        //     }
        // })
        // .collect();

        let triangles = triangulate_polygon(&contour_vertices[..contour_vertices.len() - 1]);

        println!("Triangulated contour into {} triangles", triangles.len());

        for triangle in triangles {
            vertices.push(triangle[0]);
            vertices.push(triangle[1]);
            vertices.push(triangle[2]);
        }
    }

    println!("Built filled glyph mesh with {} vertices", vertices.len());
    vertices
}
