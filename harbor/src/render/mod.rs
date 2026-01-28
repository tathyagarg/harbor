use std::collections::HashMap;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu::{self, Device};

use crate::css::r#box::Box;
use crate::css::colors::UsedColor;
use crate::css::layout::Layout;
use crate::font::otf_dtypes::GLYPH_ID;
use crate::font::tables::glyf::Point;
use crate::font::ttf::TableDirectory;
use crate::html5::dom::Document;
use crate::render::state::WindowState;
use crate::render::text::{GlyphInstance, GlyphMesh, GlyphVertex};

pub mod shapes;
pub mod state;
pub mod text;

/// Converts RGBA values (0-255 for RGB, 0-100 for A) to wgpu::Color
/// A being 0-100 is because I was feeling quirky
pub fn rgba_to_color(r: u8, g: u8, b: u8, a: u8) -> wgpu::Color {
    wgpu::Color {
        r: (r as f64) / 255.0,
        g: (g as f64) / 255.0,
        b: (b as f64) / 255.0,
        a: (a as f64) / 100.0,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub screen_size: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    pub position: [f32; 2],
    pub color: UsedColor,
}

pub fn fill_descriptor() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: (std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress),
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RendererIdentifier {
    pub font_family: String,
    pub font_weight: u16,
    pub italic: bool,
}

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

                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                for p in &points {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }

                let scale = font_size as f32 / self.font.units_per_em() as f32;

                let glyph_verts = points
                    .iter()
                    .map(|p| GlyphVertex {
                        position: [(p.x - min_x) * scale, (p.y) * scale],
                    })
                    .collect::<Vec<GlyphVertex>>();

                let glyph_mesh = GlyphMesh {
                    outline_vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Glyph Vertex Buffer"),
                        size: (glyph_verts.len() * std::mem::size_of::<GlyphVertex>()) as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    outline_vertex_count: glyph_verts.len() as u32,
                    // TODO: THIS
                    fill_vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Glyph Fill Vertex Buffer"),
                        size: 0,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    fill_vertex_count: 0,
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

                queue.write_buffer(
                    &glyph_mesh.outline_vertex_buffer,
                    0,
                    bytemuck::cast_slice(&glyph_verts),
                );

                self.glyph_cache
                    .insert((gid, font_size), glyph_mesh.clone());

                Some(glyph_mesh)
            }
        } else {
            None
        }
    }
}

#[derive(Default, Clone)]
pub struct WindowOptions {
    pub use_transparent: bool,
    pub background_color: wgpu::Color,
}

pub struct App {
    pub window_options: WindowOptions,
    pub state: Option<WindowState>,

    pub layout: Layout,

    pub document: Document,
}

impl ApplicationHandler<WindowState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("Harbor Browser")
            // TODO: Change this to not have any decorations
            .with_decorations(true);

        if self.window_options.use_transparent {
            window_attributes = window_attributes.with_transparent(true);
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(WindowState::new(
            window,
            self.window_options.clone(),
            self.layout.clone(),
            self.document.clone(),
        )));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(root) = state.layout.root_box.as_ref() {
                    let elems = Box::get_hovered_elems(root, position.x, position.y, 0.0, 0.0);

                    for (i, child) in elems.iter().enumerate() {
                        let mut child_borrow = child.borrow_mut();
                        if !child_borrow._element_state.is_hovered {
                            child_borrow.trigger_hover(&elems[..i]);
                        }
                    }

                    for (i, prev) in state.prev_hovered_elements.iter().enumerate() {
                        if !elems.contains(prev) {
                            prev.borrow_mut()
                                .leave_hover(&state.prev_hovered_elements[..i]);
                        }
                    }

                    state.prev_hovered_elements = elems;
                }
            }
            WindowEvent::RedrawRequested => {
                state.update();
                state.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state) {
                (KeyCode::Escape, ElementState::Pressed) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}
