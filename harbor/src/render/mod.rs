use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu::{self};

use crate::agent::Agent;
use crate::css::r#box::Box as CssBox;
use crate::css::colors::UsedColor;
use crate::css::layout::Layout;
use crate::globals::{
    INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH, MINIMUM_WINDOW_HEIGHT, MINIMUM_WINDOW_WIDTH,
};
use crate::html5::dom::Document;
use crate::render::state::WindowState;

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

#[derive(Default, Clone)]
pub struct WindowOptions {
    pub use_transparent: bool,
    pub background_color: wgpu::Color,
}

pub struct CallbackData {
    pub link_callback: Box<dyn Fn(&str)>,
}

pub struct App {
    pub window_options: WindowOptions,
    pub state: Option<WindowState>,

    pub agent: Option<Rc<RefCell<Agent>>>,

    pub layout: Option<Layout>,

    pub document: Option<Rc<RefCell<Document>>>,

    pub callbacks: Option<CallbackData>,
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("Harbor Browser")
            .with_decorations(false)
            .with_inner_size(winit::dpi::LogicalSize::new(
                INITIAL_WINDOW_WIDTH,
                INITIAL_WINDOW_HEIGHT,
            ))
            .with_transparent(true)
            .with_min_inner_size(winit::dpi::LogicalSize::new(
                MINIMUM_WINDOW_WIDTH,
                MINIMUM_WINDOW_HEIGHT,
            ));

        if self.window_options.use_transparent {
            window_attributes = window_attributes.with_transparent(true);
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(WindowState::new(
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
            WindowEvent::CursorMoved { position, .. } => {
                if state.layout.is_none() {
                    return;
                }

                let layout = state.layout.as_ref().unwrap();

                if let Some(root) = layout.root_box.as_ref() {
                    let elems = CssBox::get_elements_under(root, position.x, position.y, 0.0, 0.0);

                    let inner_size = state.window.inner_size();
                    let viewport_size = (inner_size.width as f64, inner_size.height as f64);

                    for (i, child) in elems.iter().enumerate() {
                        let mut child_borrow = child.borrow_mut();
                        if !child_borrow._element_state.is_hovered {
                            child_borrow.trigger_hover(&elems[..i], viewport_size);
                        }
                    }

                    for (i, prev) in state.prev_hovered_elements.iter().enumerate() {
                        if !elems.contains(prev) {
                            prev.borrow_mut()
                                .leave_hover(&state.prev_hovered_elements[..i], viewport_size);
                        }
                    }

                    state.prev_hovered_elements = elems;
                }

                state.cursor_position = (position.x, position.y);
            }
            WindowEvent::MouseInput {
                state: elem_state,
                button: MouseButton::Left,
                ..
            } => {
                if state.layout.is_none() {
                    return;
                }

                let layout = state.layout.as_ref().unwrap();

                if let Some(root) = layout.root_box.as_ref() {
                    let elems = CssBox::get_elements_under(
                        root,
                        state.cursor_position.0,
                        state.cursor_position.1,
                        0.0,
                        0.0,
                    );

                    for (i, child) in elems.iter().enumerate() {
                        let mut child_borrow = child.borrow_mut();
                        match elem_state {
                            ElementState::Pressed => {
                                child_borrow.trigger_click(&elems[..i]);
                            }
                            ElementState::Released => {
                                child_borrow.trigger_release(&elems[..i]);
                                if child_borrow.local_name == "a" {
                                    if let Some(href) = child_borrow.get_attribute("href") {
                                        if let Some(callbacks) = &self.callbacks {
                                            (callbacks.link_callback)(href);
                                        }
                                    }
                                }
                            }
                        }
                    }
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::OpenUrl(url) => {
                if let Some(agent) = &self.agent {
                    let agent_clone = Rc::clone(agent);
                    let doc = agent_clone.borrow_mut().open(&url);

                    if let Some(document) = doc {
                        self.document = Some(Rc::clone(&document));

                        let window_size = if let Some(state) = &self.state {
                            let size = state.window.inner_size();
                            (size.width as f64, size.height as f64)
                        } else {
                            (INITIAL_WINDOW_WIDTH as f64, INITIAL_WINDOW_HEIGHT as f64)
                        };

                        let layout = Layout::make_layout(Rc::clone(&document), window_size);

                        self.layout = Some(layout);
                        println!("Layout: {:#?}", self.layout.as_ref().unwrap().root_box);

                        if let Some(state) = &mut self.state {
                            state.layout = self.layout.clone();
                        }
                    }
                }
            }
        }
    }
}

pub enum AppEvent {
    OpenUrl(String),
}

impl App {
    pub fn run(&mut self) {
        let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        if let Some(agent) = &self.agent {
            let agent_clone = Rc::clone(agent);
            let doc = agent_clone
                .borrow_mut()
                .open("https://flavorless.hackclub.com/");

            if let Some(document) = doc {
                self.document = Some(Rc::clone(&document));

                let layout = Layout::make_layout(
                    Rc::clone(&document),
                    (INITIAL_WINDOW_WIDTH as f64, INITIAL_WINDOW_HEIGHT as f64),
                );

                self.layout = Some(layout);
            }
        }

        self.callbacks = Some(CallbackData {
            link_callback: Box::new(move |url: &str| {
                let _ = proxy.send_event(AppEvent::OpenUrl(url.to_string()));
            }),
        });

        let _ = event_loop.run_app(self);
    }
}
