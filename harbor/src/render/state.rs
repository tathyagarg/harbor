use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc, sync::Arc};

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{
    css::{
        r#box::{Box, BoxType},
        layout::Layout,
        properties::FontStyle,
    },
    globals::DEFAULT_FONT_FAMILY,
    html5::dom::{Element, NodeKind},
    render::{
        Globals, RendererIdentifier, WindowOptions, fill_descriptor,
        shapes::{circle_at, rectangle_at},
        text::{GlyphInstance, GlyphVertex},
    },
};

/// WindowState
/// Holds all data about the WGPU state, along with the window
pub struct WindowState {
    /// Basic WGPU state variables
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    pub layout: Option<Layout>,

    // pub viewport_width: f64,
    pub viewport_height: f64,

    pub msaa_view: wgpu::TextureView,
    pub stencil_view: wgpu::TextureView,

    pub line_render_pipeline: wgpu::RenderPipeline,
    pub glyph_fill_render_pipeline: wgpu::RenderPipeline,
    pub glyph_stencil_render_pipeline: wgpu::RenderPipeline,
    pub fill_render_pipeline: wgpu::RenderPipeline,
    pub circle_render_pipeline: wgpu::RenderPipeline,

    pub is_surface_configured: bool,

    pub window: Arc<Window>,
    pub window_options: WindowOptions,

    pub prev_hovered_elements: Vec<Rc<RefCell<Element>>>,

    pub globals_buffer: wgpu::Buffer,
    pub globals_bind_group: wgpu::BindGroup,

    pub cursor_position: (f64, f64),

    pub scroll_x: f64,
    pub scroll_y: f64,
}

impl WindowState {
    /// Renders a layout box and its children recursively
    /// # Arguments
    /// * `layout_box` - The layout box to render
    /// * `position` - The position offset to apply to the box
    /// * `parents` - A mutable reference to a vector of parent boxes and their positions
    /// * `render_pass` - A mutable reference to the current render pass
    pub fn render_box(
        &mut self,
        layout_box: Box,
        parents: &mut Vec<(Box, (f64, f64))>,
        render_pass: &mut wgpu::RenderPass,
    ) {
        let _maybe_res = (layout_box.clone(), (-self.scroll_x, -self.scroll_y));

        let respected_parent = if let Some(rel_to) = layout_box.position_relative_to {
            if rel_to > parents.len() {
                parents.last().unwrap_or(&_maybe_res)
            } else {
                &parents[parents.len() - rel_to]
            }
        } else {
            parents.first().unwrap_or(&_maybe_res)
        };

        let adj_position = (
            layout_box.position().0 + respected_parent.1.0 + layout_box.margin().left(),
            layout_box.position().1 + respected_parent.1.1 + layout_box.margin().top(),
        );

        if (adj_position.1 + layout_box._content_height < 0.0)
            || (adj_position.1 > self.window.inner_size().height as f64)
        {
            return;
        }

        let bg_color = layout_box
            .style()
            .map(|s| s.background.color().used())
            .unwrap_or([0.0, 0.0, 0.0, 0.0]);
        if bg_color[3] > 0.0 {
            render_pass.set_pipeline(&self.fill_render_pipeline);

            let window_size = self.window.inner_size();

            let x_pos = (adj_position.0 as f32 / window_size.width as f32) * 2.0 - 1.0;
            let y_pos = 1.0 - (adj_position.1 as f32 / window_size.height as f32) * 2.0;

            let pixel_w = layout_box.content_edges().horizontal() as f32;
            let pixel_h = layout_box.content_edges().vertical() as f32;

            let width = (pixel_w / window_size.width as f32) * 2.0;
            let height = (pixel_h / window_size.height as f32) * 2.0;

            let verts = rectangle_at(x_pos, y_pos, width, height, bg_color);

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

        match layout_box._box_type {
            BoxType::Block => {}
            BoxType::Inline => {
                render_pass.set_pipeline(&self.line_render_pipeline);

                if layout_box.associated_node.is_some() {
                    let node = layout_box.associated_node.as_ref().unwrap();

                    match node.borrow().deref() {
                        NodeKind::Text(text_node) => {
                            let text_content = text_node.borrow().data().to_string();
                            if text_content.trim().is_empty() {
                                return;
                            }

                            let style = parents.last().unwrap().0.style().unwrap();
                            let family = style.font.family();

                            let font_weight =
                                style.font.resolved_font_weight().unwrap_or(400) as u16;

                            let italic = matches!(style.font.style(), FontStyle::Italic);

                            if self.layout.is_none() {
                                return;
                            }

                            let layout = self.layout.as_mut().unwrap();

                            let renderer = if let Some(r) = layout
                                ._renderers
                                .get_mut(&RendererIdentifier {
                                    font_family: family
                                        .entries
                                        .first()
                                        .map(|f| f.value())
                                        .unwrap_or(DEFAULT_FONT_FAMILY.to_string()),
                                    font_weight,
                                    italic,
                                })
                                .map_or(None, |r| r.as_mut())
                            {
                                r
                            } else {
                                layout
                                    .get_renderer_mut(
                                        family
                                            .entries
                                            .first()
                                            .map(|f| f.value())
                                            .unwrap_or(DEFAULT_FONT_FAMILY.to_string()),
                                    )
                                    .expect("No renderer found for font family")
                            };

                            let mut glyph_instances: HashMap<(char, u32), Vec<GlyphInstance>> =
                                HashMap::new();

                            let mut pen_x = adj_position.0 as f32;
                            let pen_y = adj_position.1 as f32
                                + renderer.font.ascent().unwrap() as f32
                                    * (style.font.resolved_font_size().unwrap_or(16.0) as f32
                                        / renderer.font.units_per_em() as f32);

                            let font_size = style.font.resolved_font_size().unwrap_or(16.0) as f32;

                            for ch in text_content.chars() {
                                let glyph_mesh =
                                    renderer.get_from_char(ch, font_size as u32, &self.device);
                                // .cloned();

                                if let Some(glyph) = glyph_mesh {
                                    glyph_instances
                                        .entry((ch, font_size as u32))
                                        .or_default()
                                        .push(GlyphInstance {
                                            offset: [pen_x, pen_y],
                                            color: style.color.used(),
                                        });

                                    pen_x += glyph.advance_width;
                                } else {
                                    pen_x += renderer
                                        .font
                                        .advance_width(
                                            renderer.font.cmap_lookup(ch as u32).unwrap_or_else(
                                                || {
                                                    renderer
                                                        .font
                                                        .advance_width(
                                                            renderer
                                                                .font
                                                                .last_glyph_index()
                                                                .unwrap(),
                                                        )
                                                        .unwrap_or(0)
                                                },
                                            ),
                                        )
                                        .unwrap_or(0)
                                        as f32
                                        * (font_size / renderer.font.units_per_em() as f32);
                                }
                            }

                            for (key, instances) in glyph_instances {
                                let glyph =
                                    renderer.get_from_char(key.0, key.1, &self.device).unwrap();

                                let instance_buffer = self.device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Glyph Instance Buffer"),
                                        contents: bytemuck::cast_slice(&instances),
                                        usage: wgpu::BufferUsages::VERTEX
                                            | wgpu::BufferUsages::COPY_DST,
                                    },
                                );

                                render_pass.set_pipeline(&self.glyph_stencil_render_pipeline);
                                render_pass
                                    .set_vertex_buffer(0, glyph.fill_vertex_buffer.slice(..));
                                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                render_pass
                                    .draw(0..glyph.fill_vertex_count, 0..instances.len() as u32);

                                render_pass.set_pipeline(&self.glyph_fill_render_pipeline);
                                render_pass.set_stencil_reference(0);

                                render_pass
                                    .set_vertex_buffer(0, glyph.fill_vertex_buffer.slice(..));
                                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                render_pass
                                    .draw(0..glyph.fill_vertex_count, 0..instances.len() as u32);
                            }
                        }
                        _ => {}
                    }
                }
            }
            BoxType::Marker => {
                // use circle render pipeline
                render_pass.set_pipeline(&self.circle_render_pipeline);

                let window_size = self.window.inner_size();

                let pixel_x = adj_position.0 as f32;
                let pixel_y = adj_position.1 as f32;

                let x_pos = pixel_x;
                let y_pos = pixel_y;

                let radius = layout_box.content_edges().horizontal() / 2.0;

                let verts = circle_at(
                    x_pos,
                    y_pos,
                    radius as f32,
                    32,
                    [0.0, 0.0, 0.0, 1.0],
                    window_size.width as f32,
                    window_size.height as f32,
                );

                let buffer = wgpu::util::BufferInitDescriptor {
                    label: Some("Circle Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts),
                    usage: wgpu::BufferUsages::VERTEX,
                };

                let circle_vertex_buffer = self.device.create_buffer_init(&buffer);

                render_pass.set_vertex_buffer(0, circle_vertex_buffer.slice(..));
                render_pass.draw(0..verts.len() as u32, 0..1);
            }
            BoxType::None => return,
            _ => {}
        }

        parents.push((layout_box.clone(), adj_position));

        for child in &layout_box.children {
            self.render_box(child.borrow().clone(), parents, render_pass);
        }

        parents.pop();
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
            if self.layout.is_none() {
                return;
            }

            let layout = self.layout.as_ref().unwrap();

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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.stencil_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            _render_pass.set_bind_group(0, &self.globals_bind_group, &[]);

            let root_box = layout.root_box.as_ref().unwrap().borrow().clone();

            self.render_box(root_box, &mut vec![], &mut _render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub async fn new(window: Arc<Window>, window_options: WindowOptions) -> Self {
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
            sample_count: 1,
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

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Globals Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let line_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Line Render Pipeline Layout"),
                    bind_group_layouts: &[&globals_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("glyph_vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GlyphVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GlyphInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
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
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0x0,
                    write_mask: 0x0,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let glyph_stencil_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Glyph Stencil Render Pipeline"),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Glyph Stencil Render Pipeline Layout"),
                        bind_group_layouts: &[&globals_bind_group_layout],
                        push_constant_ranges: &[],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("glyph_vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<GlyphVertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<GlyphInstance>()
                                as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 1,
                                    format: wgpu::VertexFormat::Float32x2,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                    shader_location: 2,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                            ],
                        },
                    ],
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
                        write_mask: wgpu::ColorWrites::empty(),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Always,
                            pass_op: wgpu::StencilOperation::Invert,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        back: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Always,
                            pass_op: wgpu::StencilOperation::Invert,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        read_mask: 0x1,
                        write_mask: 0x1,
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                // multisample: wgpu::MultisampleState {
                //     count: 1,
                //     mask: !0,
                //     alpha_to_coverage_enabled: false,
                // },
                multiview: None,
                cache: None,
            });

        let glyph_fill_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Glyph Fill Render Pipeline Layout"),
                        bind_group_layouts: &[&globals_bind_group_layout],
                        push_constant_ranges: &[],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("glyph_vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<GlyphVertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<GlyphInstance>()
                                as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 1,
                                    format: wgpu::VertexFormat::Float32x2,
                                },
                                wgpu::VertexAttribute {
                                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                    shader_location: 2,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                            ],
                        },
                    ],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::NotEqual,
                            pass_op: wgpu::StencilOperation::Keep,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        back: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::NotEqual,
                            pass_op: wgpu::StencilOperation::Keep,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        read_mask: 0x1,
                        write_mask: 0x0,
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let fill_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fill Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[fill_descriptor()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0x1,
                    write_mask: 0x1,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let circle_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Circle Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[fill_descriptor()],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Equal,
                            pass_op: wgpu::StencilOperation::Keep,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        back: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Equal,
                            pass_op: wgpu::StencilOperation::Keep,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                        },
                        read_mask: 0x1,
                        write_mask: 0x1,
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Buffer"),
            contents: bytemuck::cast_slice(&[Globals {
                screen_size: [size.width as f32, size.height as f32],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globals Bind Group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let stencil_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Stencil Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let stencil_view = stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            surface,
            window,
            adapter,
            device,
            queue,
            config,
            msaa_view,
            stencil_view,
            layout: None,
            // viewport_width: 0.0,
            viewport_height: 0.0,
            line_render_pipeline,
            glyph_stencil_render_pipeline,
            glyph_fill_render_pipeline,
            fill_render_pipeline,
            circle_render_pipeline,
            is_surface_configured: false,
            window_options,
            prev_hovered_elements: vec![],
            globals_buffer,
            globals_bind_group,
            cursor_position: (0.0, 0.0),
            scroll_x: 0.0,
            scroll_y: 0.0,
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

            if let Some(layout) = &mut self.layout {
                layout.resized((width as f64, height as f64));

                let root_box = layout.root_box.as_ref().unwrap().borrow();

                // self.viewport_width = root_box._content_width;
                self.viewport_height = root_box._content_height;
            }

            self.queue.write_buffer(
                &self.globals_buffer,
                0,
                bytemuck::cast_slice(&[Globals {
                    screen_size: [width as f32, height as f32],
                }]),
            );

            let msaa_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Multisampled Texture"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let stencil_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Stencil Texture"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.stencil_view =
                stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }
}
