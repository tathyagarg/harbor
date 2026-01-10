#![allow(warnings)]

use std::sync::Arc;

use wgpu::util::DeviceExt;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu;

use crate::font::tables::glyf::Point;

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
                    view: &view,
                    resolve_target: None,
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
            _render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _render_pass.draw(0..self.number_of_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub async fn new(window: Arc<Window>, window_options: WindowOptions, verts: &[Vertex]) -> Self {
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let number_of_vertices = verts.len() as u32;

        Self {
            surface,
            window,
            adapter,
            device,
            queue,
            config,
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
            // self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = false;
        }
    }
}

#[derive(Default, Clone)]
pub struct WindowOptions {
    pub use_transparent: bool,
    pub background_color: wgpu::Color,
}

#[derive(Default)]
pub struct App {
    pub window_options: WindowOptions,
    pub state: Option<State>,

    pub vertices: Vec<Vertex>,
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

        self.state = Some(pollster::block_on(State::new(
            window,
            self.window_options.clone(),
            &self.vertices,
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
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
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
        width: f32,
        height: f32,
        color: [f32; 3],
    ) -> Vertex {
        let vertex_position = point.vertex_position(origin, scale);

        Vertex {
            position: [
                (vertex_position[0] / width) * 2.0 - 1.0,
                1.0 - (vertex_position[1] / height) * 2.0,
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
    width: f32,
    height: f32,
    color: [f32; 3],
}

impl VertexMaker {
    pub fn new(origin: (f32, f32), scale: f32, width: f32, height: f32, color: [f32; 3]) -> Self {
        Self {
            origin,
            scale,
            width,
            height,
            color,
        }
    }

    pub fn from_point(&self, point: &Point) -> Vertex {
        Vertex::clipped_from_point(
            point,
            self.origin,
            self.scale,
            self.width,
            self.height,
            self.color,
        )
    }
}

pub enum Segment {
    Line(Vertex, Vertex),
    Quadratic(Vertex, Vertex, Vertex),
    Cubic(Vertex, Vertex, Vertex, Vertex),
}

impl Segment {
    pub fn flatten(&self, out: &mut Vec<Vertex>, precision: f32) {
        match self {
            Segment::Line(v0, v1) => {
                out.push(v0.clone());
                out.push(v1.clone());
            }
            Segment::Quadratic(v0, c, v2) => {
                if c.distance_to_line(v0, v2) < precision {
                    out.push(v0.clone());
                    out.push(v2.clone());
                } else {
                    let mid1 = Vertex::midpoint(v0, c);
                    let mid2 = Vertex::midpoint(c, v2);
                    let mid = Vertex::midpoint(&mid1, &mid2);

                    Segment::Quadratic(v0.clone(), mid1, mid.clone()).flatten(out, precision);
                    Segment::Quadratic(mid, mid2, v2.clone()).flatten(out, precision);
                }
            }
            Segment::Cubic(v0, v1, v2, v3) => {
                panic!("Cubic segment flattening not implemented yet.");
                // Simple linear approximation for demonstration
            }
        }
    }
}
