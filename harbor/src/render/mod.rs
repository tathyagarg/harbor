// #![allow(warnings)]

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu;

use crate::css::r#box::{Box, BoxType};
use crate::css::colors::UsedColor;
use crate::css::layout::Layout;
use crate::font::ttf::ParsedTableDirectory;
use crate::html5::dom::Document;

pub mod shapes;
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
                contents: bytemuck::cast_slice(&[] as &[text::Vertex]),
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
    pub vertex_cache: HashMap<(String, u32, u32, u32), (Vec<text::Vertex>, f32)>,

    pub window_size: (f32, f32),

    _empty_buffer: Option<wgpu::Buffer>,
    outline_vertex_buffers: HashMap<(String, u32, u32, u32), (wgpu::Buffer, usize)>,
}

impl TextRenderer {
    pub fn new() -> TextRendererCreator {
        TextRendererCreator::default()
    }

    pub fn vertices(
        &mut self,
        text: String,
        color: UsedColor,
        font_size: f32,
        position: (u32, u32),
    ) -> Vec<text::Vertex> {
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

        let verts = font.rasterize(
            text.as_str(),
            color,
            scale,
            800.0 / font_size,
            float_position,
            self.window_size,
        );

        if update_cache {
            self.vertex_cache.insert(
                (text.clone(), position.0, position.1, font_size as u32),
                (verts.clone(), font_size),
            );

            self.outline_vertex_buffers
                .remove(&(text, position.0, position.1, font_size as u32));
        }

        verts
    }

    pub fn update_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        text: String,
        color: UsedColor,
        font_size: f32,
        position: (u32, u32),
    ) {
        let verts = self.vertices(text.clone(), color, font_size, position);

        let key = (text, position.0, position.1, font_size as u32);

        let existing_buffer = self.outline_vertex_buffers.get_mut(&key);

        let new_buffer = if let Some((_, _)) = existing_buffer {
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

/// WindowState
/// Holds all data about the WGPU state, along with the window
#[allow(dead_code)]
pub struct WindowState {
    /// Basic WGPU state variables
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    layout: Layout,

    msaa_view: wgpu::TextureView,

    line_render_pipeline: wgpu::RenderPipeline,
    fill_render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    number_of_vertices: u32,

    is_surface_configured: bool,

    window: Arc<Window>,
    window_options: WindowOptions,

    document: Document,
}

impl WindowState {
    pub fn render_box(
        &mut self,
        layout_box: Box,
        position: (f64, f64),
        render_pass: &mut wgpu::RenderPass,
    ) {
        match layout_box._box_type {
            BoxType::Block => {
                render_pass.set_pipeline(&self.fill_render_pipeline);
                let bg_color = layout_box.associated_style.background.color().used();

                if bg_color[3] > 0.0 {
                    let window_size = self.window.inner_size();

                    // println!("Box: {:#?}", layout_box);

                    let pixel_x = layout_box.position().0 as f32 + position.0 as f32;
                    let pixel_y = layout_box.position().1 as f32 + position.1 as f32;

                    let x_pos = (pixel_x / window_size.width as f32) * 2.0 - 1.0;
                    let y_pos = -((pixel_y / window_size.height as f32) * 2.0 - 1.0);

                    let pixel_w = layout_box.content_edges().horizontal() as f32;
                    let pixel_h = layout_box.content_edges().vertical() as f32;

                    let width = (pixel_w / window_size.width as f32) * 2.0;
                    let height = (pixel_h / window_size.height as f32) * 2.0;

                    let verts = shapes::rectangle_at(x_pos, y_pos, width, height, bg_color);

                    // println!("verts: {:#?}", verts);

                    let bg_vertex_buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Background Vertex Buffer"),
                                contents: bytemuck::cast_slice(&verts),
                                usage: wgpu::BufferUsages::VERTEX,
                            });

                    render_pass.set_vertex_buffer(0, bg_vertex_buffer.slice(..));
                    render_pass.draw(0..verts.len() as u32, 0..1);
                }
            }
            BoxType::Inline => {
                render_pass.set_pipeline(&self.line_render_pipeline);
                let adj_position = (
                    layout_box.position().0 as f64 + position.0,
                    layout_box.position().1 as f64 + position.1,
                );

                if layout_box.associated_node.is_some() {
                    let node = layout_box.associated_node.as_ref().unwrap();

                    match node.borrow().deref() {
                        crate::html5::dom::NodeKind::Text(text_node) => {
                            let text_content = text_node.borrow().data().to_string();

                            if text_content.trim().is_empty() {
                                return;
                            }

                            let family = layout_box.associated_style.font.family();
                            let mut font_iter = family.entries.iter();
                            let mut renderer = loop {
                                if let Some(font_family) = font_iter.next() {
                                    if let Some(renderer_option) =
                                        self.layout._renderers.get_mut(&font_family.value())
                                    {
                                        if let Some(renderer) = renderer_option {
                                            break renderer.clone();
                                        }
                                    }
                                } else {
                                    // Fallback to default font
                                    if let Some(renderer_option) =
                                        self.layout._renderers.get_mut("Times New Roman")
                                    {
                                        if let Some(renderer) = renderer_option {
                                            break renderer.clone();
                                        }
                                    } else {
                                        return;
                                    }
                                }
                            };

                            let font_size = layout_box
                                .associated_style
                                .font
                                .resolved_font_size()
                                .unwrap_or(16.0) as f32;

                            let verts = renderer.vertices(
                                text_content.clone(),
                                layout_box.associated_style.color.used(),
                                font_size,
                                (adj_position.0 as u32, adj_position.1 as u32),
                            );

                            if !verts.is_empty() {
                                renderer.update_vertex_buffer(
                                    &self.device,
                                    text_content.clone(),
                                    layout_box.associated_style.color.used(),
                                    font_size,
                                    (adj_position.0 as u32, adj_position.1 as u32),
                                );

                                let key = (
                                    text_content.clone(),
                                    adj_position.0 as u32,
                                    adj_position.1 as u32,
                                    font_size as u32,
                                );

                                if let Some((buffer, count)) =
                                    renderer.outline_vertex_buffers.get(&key)
                                {
                                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                                    render_pass.draw(0..*count as u32, 0..1);
                                }
                            }
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

            self.render_box(child.borrow().clone(), new_position, render_pass);
        }
    }

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

            let root_box = self.layout.root_box.as_ref().unwrap().borrow().clone();

            self.render_box(root_box, (0.0, 0.0), &mut _render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub async fn new(
        window: Arc<Window>,
        window_options: WindowOptions,
        layout: Layout,
        document: Document,
    ) -> Self {
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

        let line_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[text::Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
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

        let fill_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fill Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[text::Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            contents: bytemuck::cast_slice(&[] as &[text::Vertex]),
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
            line_render_pipeline,
            fill_render_pipeline,
            vertex_buffer,
            number_of_vertices,
            is_surface_configured: false,
            window_options,
            document,
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
