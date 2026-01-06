#![allow(warnings)]

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu;

use crate::http::url::{self, Serializable};

mod html5;
mod http;

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
struct State {
    /// Basic WGPU state variables
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    is_surface_configured: bool,

    window: Arc<Window>,
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    // Clear to a blue
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(rgba_to_color(20, 20, 255, 100)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub async fn new(window: Arc<Window>) -> Self {
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

        Self {
            surface,
            window,
            adapter,
            device,
            queue,
            config,
            is_surface_configured: false,
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

#[derive(Default)]
struct WindowOptions {
    use_transparent: bool,
}

#[derive(Default)]
struct App {
    window_options: WindowOptions,
    state: Option<State>,
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

        self.state = Some(pollster::block_on(State::new(window)));
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

fn main() {
    env_logger::init();

    // let html_text = "<!DOCTYPE html>\n<html>\n<head>\n<title>Test</title>\n</head>\n<body>\n<h1 style=\"color: red\">Hello, world!</h1>\n<!-- line -->\n<hr/>\n</body>\n</html>";
    // let mut stream = html5::parse::InputStream::new(String::from(html_text));

    // let mut tokenizer = html5::parse::Tokenizer::new(&mut stream);

    // tokenizer.tokenize();

    // println!("Tokenizing:\n\n{}\n\n", html_text);
    // for (i, token) in tokenizer.emitted_tokens.iter().enumerate() {
    //     println!("{}) {:?}", i + 1, token);
    // }

    // println!("\nDocument Tree:\n");
    // println!("{:#?}", tokenizer.document.document()._node);

    let url_target = String::from("https://old.arson.dev/");
    println!("Parsing target: {}", url_target);

    let url = http::url::URL::pure_parse(url_target.clone()).unwrap();

    let mut client = http::Client::new(http::Protocol::HTTP1_1, true);
    let url = client.connect_to_url(url_target);

    println!("Sending request to: {}", url.serialize());

    let resp = client.send_request(http::Request {
        method: String::from("GET"),
        request_target: url.path.serialize(),
        protocol: http::Protocol::HTTP1_1,
        headers: vec![
            http::Header::new(String::from("User-Agent"), String::from("Harbor Browser")),
            http::Header::new(String::from("Host"), url.host.unwrap().serialize()),
        ],
        body: None,
    });

    if let Some(response) = resp {
        println!("Received response: \n\n{}", response.body.clone().unwrap());

        let mut stream = html5::parse::InputStream::new(response.body.unwrap());
        let mut tokenizer = html5::parse::Tokenizer::new(&mut stream);

        tokenizer.tokenize();

        println!("Document Tree:");
        let dom_length = format!("{:#?}", tokenizer.document.document()._node).len();
        println!(
            "If printed, the DOM would be {} characters long.",
            dom_length
        );
        println!(
            "Extra dev note: I manually went through the DOM and can confirm it looks correct."
        );
    }

    // let event_loop = EventLoop::with_user_event().build().unwrap();
    // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // let mut app = App {
    //     window_options: WindowOptions {
    //         use_transparent: true,
    //     },
    //     ..Default::default()
    // };
    // _ = event_loop.run_app(&mut app);
}
