#![allow(warnings)]

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu;

use crate::css::r#box::{Box, BoxType};
use crate::css::layout::Layout;
use crate::font::otf_dtypes::int16;
use crate::font::tables::glyf::{GlyphTransform, Point};
use crate::font::ttf::{ParsedTableDirectory, TableDirectory};
use crate::html5::dom::Text;

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

#[derive(Clone, Default)]
pub struct TextRendererCreator {
    pub font: Option<ParsedTableDirectory>,
    pub window_size: (f32, f32),

    pub buffer: Option<wgpu::Buffer>,

    pub color: [f32; 3],
}

impl TextRendererCreator {
    pub fn with_font(mut self, font: ParsedTableDirectory) -> Self {
        self.font = Some(font);
        self
    }

    pub fn with_window_size(mut self, window_size: (f32, f32)) -> Self {
        self.window_size = window_size;
        self
    }

    pub fn with_device(mut self, device: &wgpu::Device) -> Self {
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&[] as &[Vertex]),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );
        self
    }

    pub fn with_color(mut self, color: [f32; 3]) -> Self {
        self.color = color;
        self
    }

    pub fn build(self) -> TextRenderer {
        TextRenderer {
            font: self.font,
            window_size: self.window_size,
            vertex_cache: HashMap::new(),
            _empty_buffer: self.buffer,
            outline_vertex_buffers: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct TextRenderer {
    pub font: Option<ParsedTableDirectory>,

    /// Key: (text, x position, y position, font-size (rounded))
    /// Value: (vertices, font-size)
    /// The font size in the key is rounded
    /// When getting a value from the cache, if the required font-size matches the key but is not
    /// equal, the cache will not be used and new vertices will be generated. This will not update
    /// the cache.
    pub vertex_cache: HashMap<(String, u32, u32, u32), (Vec<Vertex>, f32)>,

    pub window_size: (f32, f32),

    _empty_buffer: Option<wgpu::Buffer>,
    outline_vertex_buffers: HashMap<(String, u32, u32, u32), (wgpu::Buffer, usize)>,
}

impl TextRenderer {
    pub fn new() -> TextRendererCreator {
        TextRendererCreator::default()
    }

    pub fn vertices(&mut self, text: String, font_size: f32, position: (u32, u32)) -> Vec<Vertex> {
        println!(
            "Generating vertices for text: '{}', font_size: {}, position: {:?}",
            text, font_size, position
        );
        let update_cache = if let Some((verts, cached_font_size)) =
            self.vertex_cache
                .get(&(text.clone(), position.0, position.1, font_size as u32))
        {
            if *cached_font_size == font_size {
                return verts.clone();
            }

            false
        } else {
            true
        };

        let font = match &self.font {
            Some(f) => f,
            None => return vec![],
        };

        let scale = font_size / font.units_per_em() as f32;

        let mut y = position.1 as f32;
        y += (font.ascent().unwrap_or(0) as f32) * scale;

        let float_position = (position.0 as f32, y);

        println!("Rasterizing");
        let verts = font.rasterize(
            text.as_str(),
            scale,
            800.0 / font_size,
            float_position,
            self.window_size,
        );

        if update_cache {
            println!("Updating vertex cache");
            self.vertex_cache.insert(
                (text.clone(), position.0, position.1, font_size as u32),
                (verts.clone(), font_size),
            );

            self.outline_vertex_buffers
                .remove(&(text, position.0, position.1, font_size as u32));
        }

        println!("Generated {} vertices", verts.len());
        verts
    }

    pub fn update_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text: String,
        font_size: f32,
        position: (u32, u32),
    ) {
        let verts = self.vertices(text.clone(), font_size, position);

        let key = (text, position.0, position.1, font_size as u32);

        let existing_buffer = self.outline_vertex_buffers.get_mut(&key);

        let new_buffer = if let Some((buffer, _)) = existing_buffer {
            return;
        } else {
            let new_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

            new_buffer
        };

        self.outline_vertex_buffers
            .insert(key, (new_buffer.clone(), verts.len()));
    }

    pub fn resized(&mut self, new_size: (f32, f32)) {
        self.window_size = new_size;
        self.vertex_cache.clear();
    }
}

/// State
/// Holds all data about the WGPU state, along with the window
#[allow(dead_code)]
pub struct State {
    /// Basic WGPU state variables
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    layout: Layout,

    msaa_view: wgpu::TextureView,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    number_of_vertices: u32,

    is_surface_configured: bool,

    window: Arc<Window>,
    window_options: WindowOptions,
}

impl State {
    pub fn render(&mut self) {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return;
        }

        let output = match self.surface.get_current_texture() {
            Ok(out) => out,
            // Resizing also guarantees that the surface is configured correctly.
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                return;
            }
            Err(_) => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_view,
                    resolve_target: Some(&view),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.window_options.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            _render_pass.set_pipeline(&self.render_pipeline);

            if self.number_of_vertices > 0 {
                _render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                _render_pass.draw(0..self.number_of_vertices, 0..1);
            }

            fn render_box(
                layout_box: Box,
                position: (f64, f64),
                layout: &mut Layout,
                device: &wgpu::Device,
                queue: &wgpu::Queue,
                render_pass: &mut wgpu::RenderPass<'_>,
            ) {
                match layout_box._box_type {
                    BoxType::Block => {}
                    BoxType::Inline => {
                        if layout_box.associated_node.is_some() {
                            let node = layout_box.associated_node.as_ref().unwrap();

                            match node.borrow().deref() {
                                crate::html5::dom::NodeKind::Text(text_node) => {
                                    let text_content = text_node.borrow().data().to_string();

                                    if text_content.trim().is_empty() {
                                        return;
                                    }

                                    println!("Rendering text: {}", text_content);
                                    layout
                                        ._renderers
                                        .get_mut(
                                            &layout_box
                                                ._font_family
                                                .clone()
                                                .unwrap_or("Times New Roman".to_string()),
                                        )
                                        .and_then(|renderer_option| {
                                            println!(
                                                "Using renderer for font: {:?}",
                                                layout_box._font_family
                                            );
                                            if let Some(renderer) = renderer_option {
                                                println!("Renderer found, generating vertices");
                                                let font_size =
                                                    layout_box._font_size.unwrap_or(16.0) as f32;

                                                let verts = renderer.vertices(
                                                    text_content.clone(),
                                                    font_size,
                                                    (position.0 as u32, position.1 as u32),
                                                );
                                                println!("Generated {} vertices", verts.len());

                                                if !verts.is_empty() {
                                                    println!(
                                                        "Updating vertex buffer for text: {}",
                                                        text_content
                                                    );
                                                    renderer.update_vertex_buffer(
                                                        device,
                                                        queue,
                                                        text_content.clone(),
                                                        font_size,
                                                        (position.0 as u32, position.1 as u32),
                                                    );

                                                    let key = (
                                                        text_content.clone(),
                                                        position.0 as u32,
                                                        position.1 as u32,
                                                        font_size as u32,
                                                    );

                                                    if let Some((buffer, count)) =
                                                        renderer.outline_vertex_buffers.get(&key)
                                                    {
                                                        println!(
                                                            "Drawing text buffer for key: {:?}",
                                                            key
                                                        );
                                                        render_pass
                                                            .set_vertex_buffer(0, buffer.slice(..));
                                                        render_pass.draw(0..*count as u32, 0..1);
                                                    }
                                                }
                                            }

                                            Some(())
                                        });
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }

                for child in &layout_box.children {
                    let new_position = (
                        layout_box.position().0 + position.0,
                        layout_box.position().1 + position.1,
                    );

                    render_box(
                        child.borrow().clone(),
                        new_position,
                        layout,
                        device,
                        queue,
                        render_pass,
                    );
                }
            }

            let root_box = self.layout.root_box.as_ref().unwrap().borrow().clone();

            render_box(
                root_box,
                (0.0, 0.0),
                &mut self.layout,
                &self.device,
                &self.queue,
                &mut _render_pass,
            );

            // if !self.layout.text.is_empty() {
            //     for entry in self.layout.text.iter() {
            //         if let Some(renderer) = self.layout._renderers.get_mut(&entry.font_name) {
            //             if let Some(r) = renderer {
            //                 let verts = r.vertices(
            //                     entry.content.clone(),
            //                     entry.font_size,
            //                     (entry.origin.0 as u32, entry.origin.1 as u32),
            //                 );

            //                 if !verts.is_empty() {
            //                     r.update_vertex_buffer(
            //                         &self.device,
            //                         &self.queue,
            //                         entry.content.clone(),
            //                         entry.font_size,
            //                         (entry.origin.0 as u32, entry.origin.1 as u32),
            //                     );

            //                     let key = (
            //                         entry.content.clone(),
            //                         entry.origin.0 as u32,
            //                         entry.origin.1 as u32,
            //                         entry.font_size as u32,
            //                     );

            //                     if let Some((buffer, count)) = r.outline_vertex_buffers.get(&key) {
            //                         _render_pass.set_vertex_buffer(0, buffer.slice(..));
            //                         _render_pass.draw(0..*count as u32, 0..1);
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub async fn new(window: Arc<Window>, window_options: WindowOptions, layout: Layout) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),

                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Multisampled Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&[] as &[Vertex]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let number_of_vertices = 0;

        let window_size = (size.width as f32, size.height as f32);

        let mut populated_layout = layout.clone();
        populated_layout.populate_renderers(window_size);

        Self {
            surface,
            window,
            adapter,
            device,
            queue,
            config,
            msaa_view,
            layout: populated_layout,
            render_pipeline,
            vertex_buffer,
            number_of_vertices,
            is_surface_configured: false,
            window_options,
        }
    }

    pub fn update(&mut self) {
        if !self.is_surface_configured {
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            self.is_surface_configured = false;

            self.layout.resized((width as f64, height as f64));

            let msaa_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Multisampled Texture"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }
}

#[derive(Default, Clone)]
pub struct WindowOptions {
    pub use_transparent: bool,
    pub background_color: wgpu::Color,
}

#[derive(Clone)]
pub struct TextEntry {
    pub font_name: String,
    pub origin: (f32, f32),

    pub content: String,

    pub font_size: f32,
}

// #[derive(Clone)]
// pub struct Layout {
//     pub available_fonts: HashMap<String, ParsedTableDirectory>,
//     _renderers: HashMap<String, Option<TextRenderer>>,
//
//     pub text: Vec<TextEntry>,
// }
//
// impl Layout {
//     pub fn new(fonts: HashMap<String, ParsedTableDirectory>, text: Vec<TextEntry>) -> Self {
//         let renderers = fonts.iter().map(|(name, _)| (name.clone(), None)).collect();
//
//         Self {
//             available_fonts: fonts,
//             _renderers: renderers,
//             text,
//         }
//     }
//
//     fn resized(&mut self, new_size: (f32, f32)) {
//         for renderer in self._renderers.values_mut() {
//             if let Some(r) = renderer {
//                 r.resized(new_size);
//             }
//         }
//     }
//
//     fn populate_renderers(&mut self, device: &wgpu::Device, window_size: (f32, f32)) {
//         for entry in &self.text {
//             if let Some(font) = self.available_fonts.get(&entry.font_name) {
//                 if self._renderers.get(&entry.font_name).is_none()
//                     || self._renderers.get(&entry.font_name).unwrap().is_none()
//                 {
//                     self._renderers.insert(
//                         entry.font_name.clone(),
//                         Some(
//                             TextRenderer::new()
//                                 .with_font(font.clone())
//                                 .with_window_size(window_size)
//                                 .with_color([0.0, 0.0, 0.0])
//                                 .build(),
//                         ),
//                     );
//                 }
//             } else {
//                 panic!("Unknown font name: {}", entry.font_name);
//             }
//         }
//     }
// }

pub struct App {
    pub window_options: WindowOptions,
    pub state: Option<State>,

    pub layout: Layout,
}

impl ApplicationHandler<State> for App {
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
        let inner_size = window.inner_size();

        self.state = Some(pollster::block_on(State::new(
            window,
            self.window_options.clone(),
            self.layout.clone(),
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
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    pub fn clipped_from_point(
        point: &Point,
        origin: (f32, f32),
        scale: f32,
        window_size: (f32, f32),
        color: [f32; 3],
    ) -> Vertex {
        let vertex_position = point.vertex_position(origin, scale);

        Vertex {
            position: [
                (vertex_position[0] / window_size.0) * 2.0 - 1.0,
                1.0 - (vertex_position[1] / window_size.1) * 2.0,
                vertex_position[2],
            ],
            color,
        }
    }

    pub fn distance_to(&self, other: &Vertex) -> f32 {
        let dx = self.position[0] - other.position[0];
        let dy = self.position[1] - other.position[1];
        let dz = self.position[2] - other.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn distance_to_line(&self, v1: &Vertex, v2: &Vertex) -> f32 {
        let a = self.position[0] - v1.position[0];
        let b = self.position[1] - v1.position[1];
        let c = self.position[2] - v1.position[2];

        let d = v2.position[0] - v1.position[0];
        let e = v2.position[1] - v1.position[1];
        let f = v2.position[2] - v1.position[2];

        let dot = a * d + b * e + c * f;
        let len_sq = d * d + e * e + f * f;
        let param = if len_sq != 0.0 { dot / len_sq } else { -1.0 };

        let (xx, yy, zz) = if param < 0.0 {
            (v1.position[0], v1.position[1], v1.position[2])
        } else if param > 1.0 {
            (v2.position[0], v2.position[1], v2.position[2])
        } else {
            (
                v1.position[0] + param * d,
                v1.position[1] + param * e,
                v1.position[2] + param * f,
            )
        };

        let dx = self.position[0] - xx;
        let dy = self.position[1] - yy;
        let dz = self.position[2] - zz;

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn midpoint(v1: &Vertex, v2: &Vertex) -> Vertex {
        Vertex {
            position: [
                (v1.position[0] + v2.position[0]) / 2.0,
                (v1.position[1] + v2.position[1]) / 2.0,
                (v1.position[2] + v2.position[2]) / 2.0,
            ],
            color: v1.color,
        }
    }

    pub fn to_clip(&self, width: f32, height: f32) -> Vertex {
        Vertex {
            position: [
                (self.position[0] / width) * 2.0 - 1.0,
                1.0 - (self.position[1] / height) * 2.0,
                self.position[2],
            ],
            color: self.color,
        }
    }
}

pub struct VertexMaker {
    origin: (f32, f32),
    scale: f32,
    window_size: (f32, f32),
    color: [f32; 3],
}

impl VertexMaker {
    pub fn new(origin: (f32, f32), scale: f32, window_size: (f32, f32), color: [f32; 3]) -> Self {
        Self {
            origin,
            scale,
            window_size,
            color,
        }
    }

    pub fn from_point(&self, point: &Point) -> Vertex {
        Vertex::clipped_from_point(point, self.origin, self.scale, self.window_size, self.color)
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

    pub fn translate(&mut self, dx: int16, dy: int16) {
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
