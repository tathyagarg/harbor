use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu::{self};

use crate::css::r#box::Box as CssBox;
use crate::css::colors::UsedColor;
use crate::css::layout::Layout;
use crate::css::properties::Resolvable;
use crate::globals::{
    ADDRESS_BAR_ADDRESS_OFFSET, ADDRESS_BAR_OFFSET, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH,
    MINIMUM_WINDOW_HEIGHT, MINIMUM_WINDOW_WIDTH, NEW_TAB_URL, TAB_WIDTH, TABS_BAR_OFFSET,
    TOOLBAR_OFFSET,
};
use crate::html5::dom::Document;
use crate::html5::elements::anchor::AnchorElement;
use crate::render::state::{TabData, WindowState};
use crate::user_agent::Agent;

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
    pub open_tab: Box<dyn Fn(&TabData)>,
}

pub struct App {
    pub window_options: WindowOptions,
    pub state: Option<WindowState>,

    pub agent: Option<Rc<RefCell<Agent>>>,

    pub document: Option<Rc<RefCell<Document>>>,

    pub callbacks: Option<CallbackData>,
}

impl App {
    pub fn new(window_options: WindowOptions, agent: Option<Rc<RefCell<Agent>>>) -> Self {
        Self {
            window_options,
            state: None,
            agent,
            document: None,
            callbacks: None,
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    // Initialization
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("Harbor Browser")
            .with_decorations(true)
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
            Some(NEW_TAB_URL.to_string()),
        )));

        if let Some(agent) = &self.agent {
            let agent_clone = Rc::clone(agent);

            let state = self.state.as_mut().unwrap();

            let tab_data = state.tab_datas[state.active_tab].clone();

            let doc = agent_clone.borrow_mut().open(&tab_data.url);
            state.tab_datas[state.active_tab].document = doc.clone();

            if let Some(document) = doc {
                self.document = Some(Rc::clone(&document));

                let layout = Layout::make_layout(
                    Rc::clone(&document),
                    (INITIAL_WINDOW_WIDTH as f64, INITIAL_WINDOW_HEIGHT as f64),
                );

                state.layout = Some(layout.clone());

                let rc = Rc::new(RefCell::new(layout));
                state.tab_datas[state.active_tab].layout = Some(rc);
                state.address_bar_input = tab_data.url.clone();
            }
        }
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
            WindowEvent::ModifiersChanged(m) => state.modifiers = m,
            WindowEvent::CursorMoved { position, .. } => {
                if state.layout.is_none() {
                    return;
                }

                let layout = state.layout.as_ref().unwrap();
                let tab_data = state.tab_datas.get(state.active_tab).unwrap();

                if let Some(root) = layout.root_box.as_ref() {
                    let toolbar =
                        TOOLBAR_OFFSET(state.config.width as f64, state.config.height as f64);

                    let elems = CssBox::get_elements_under(
                        root,
                        position.x - toolbar.0,
                        position.y - toolbar.1,
                        -tab_data.scroll_x,
                        -tab_data.scroll_y,
                    );

                    let inner_size = state.window.inner_size();
                    let viewport_size = (inner_size.width as f64, inner_size.height as f64);

                    for (i, child) in elems.iter().enumerate() {
                        let mut child_borrow = child.borrow_mut();
                        if !child_borrow._element_state.is_hovered {
                            child_borrow.trigger_hover(&elems[..i], viewport_size);

                            state
                                .window
                                .set_cursor(child_borrow.style().cursor.resolved());
                        }
                    }

                    for (i, prev) in state.prev_hovered_elements.iter().enumerate() {
                        if !elems.contains(prev) {
                            prev.borrow_mut()
                                .leave_hover(&state.prev_hovered_elements[..i], viewport_size);

                            state
                                .window
                                .set_cursor(prev.borrow().style().cursor.resolved());
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
                let tab_data = state.tab_datas.get(state.active_tab).unwrap();

                let tabs_bar_offset =
                    TABS_BAR_OFFSET(state.config.width as f64, state.config.height as f64);

                let address_bar_offset =
                    ADDRESS_BAR_OFFSET(state.config.width as f64, state.config.height as f64);

                let address_bar_address_offset = ADDRESS_BAR_ADDRESS_OFFSET(
                    state.config.width as f64,
                    state.config.height as f64,
                );

                let toolbar_offset =
                    TOOLBAR_OFFSET(state.config.width as f64, state.config.height as f64);

                if state.cursor_position.1 < tabs_bar_offset.1 {
                    if elem_state == ElementState::Pressed {
                        let selected_tab_index = (state.cursor_position.0
                            / TAB_WIDTH(state.config.width as f64, state.tab_datas.len()))
                        .floor() as usize;

                        if selected_tab_index >= state.tab_datas.len() {
                            return;
                        }

                        state.active_tab = selected_tab_index;

                        if let Some(callbacks) = &self.callbacks {
                            let tab_data = &state.tab_datas[state.active_tab];
                            (callbacks.open_tab)(tab_data);
                        }
                    }
                } else if state.cursor_position.1 < tabs_bar_offset.1 + address_bar_offset.1 {
                    if state.cursor_position.0 > address_bar_address_offset.0 {
                        if elem_state == ElementState::Pressed {
                            state.address_bar_active = true;
                        }
                    } else {
                        if elem_state == ElementState::Pressed {
                            let item_pressed = (state.cursor_position.0
                                / (address_bar_address_offset.0 / 3.0))
                                .floor() as usize;

                            let tab_data = state.tab_datas.get_mut(state.active_tab).unwrap();

                            match item_pressed {
                                0 => {
                                    let prev_url = tab_data.prev_urls.pop();
                                    if prev_url.is_none() {
                                        return;
                                    }

                                    let prev_url = prev_url.unwrap();
                                    let curr_url = tab_data.url.clone();

                                    tab_data.next_urls.push(curr_url);

                                    if let Some(callbacks) = &self.callbacks {
                                        (callbacks.link_callback)(&prev_url);
                                    }
                                }
                                1 => {
                                    tab_data.document = None;
                                    tab_data.layout = None;
                                    tab_data.scroll_x = 0.0;
                                    tab_data.scroll_y = 0.0;

                                    if let Some(callbacks) = &self.callbacks {
                                        (callbacks.open_tab)(tab_data);
                                    }
                                }
                                2 => {
                                    let next_url = tab_data.next_urls.pop();
                                    if next_url.is_none() {
                                        return;
                                    }

                                    let next_url = next_url.unwrap();
                                    let curr_url = tab_data.url.clone();

                                    tab_data.prev_urls.push(curr_url);

                                    if let Some(callbacks) = &self.callbacks {
                                        (callbacks.link_callback)(&next_url);
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                } else {
                    if let Some(root) = layout.root_box.as_ref() {
                        let elems = CssBox::get_elements_under(
                            root,
                            state.cursor_position.0 - toolbar_offset.0,
                            state.cursor_position.1 - toolbar_offset.1,
                            -tab_data.scroll_x,
                            -tab_data.scroll_y,
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
                                        let anchor =
                                            AnchorElement::from_element(child_borrow.deref());

                                        if let Some(hyperlink_utils) = anchor.hyperlink_utils {
                                            if let Some(callbacks) = &self.callbacks {
                                                (callbacks.link_callback)(&hyperlink_utils.href);
                                            }
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
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x as f64 * 20.0, y as f64 * 20.0),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x, pos.y),
                };

                let tab_data = state.tab_datas.get_mut(state.active_tab).unwrap();

                tab_data.scroll_x = (tab_data.scroll_x - delta_x).max(0.0);
                tab_data.scroll_y = (tab_data.scroll_y - delta_y).max(0.0);
                // .min(state.viewport_height - state.window.inner_size().height as f64);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        logical_key,
                        ..
                    },
                ..
            } => {
                if !state.address_bar_active {
                    match (code, key_state) {
                        (KeyCode::Escape, ElementState::Pressed) => event_loop.exit(),
                        (KeyCode::KeyT, ElementState::Pressed) => {
                            if cfg!(target_os = "macos") {
                                if !state.modifiers.state().super_key() {
                                    return;
                                }
                            } else {
                                if !state.modifiers.state().control_key() {
                                    return;
                                }
                            }

                            if let Some(state) = &mut self.state {
                                state
                                    .tab_datas
                                    .insert(state.active_tab + 1, TabData::empty_from(NEW_TAB_URL));

                                state.active_tab += 1;

                                if let Some(callbacks) = &self.callbacks {
                                    let tab_data = &state.tab_datas[state.active_tab];
                                    (callbacks.open_tab)(tab_data);
                                }
                            }
                        }
                        (KeyCode::Slash, ElementState::Pressed) => {
                            state.address_bar_active = true;
                        }
                        (KeyCode::Tab, ElementState::Pressed) => {
                            if !state.modifiers.state().alt_key() {
                                return;
                            }

                            let direction = if state.modifiers.state().shift_key() {
                                -1
                            } else {
                                1
                            };

                            if key_state == ElementState::Pressed {
                                let num_tabs = state.tab_datas.len();

                                if num_tabs == 0 {
                                    return;
                                }

                                state.active_tab = ((state.active_tab as isize + direction)
                                    .rem_euclid(num_tabs as isize))
                                    as usize;

                                if let Some(callbacks) = &self.callbacks {
                                    let tab_data = &state.tab_datas[state.active_tab];
                                    (callbacks.open_tab)(tab_data);
                                }
                            }
                        }
                        (KeyCode::KeyW, ElementState::Pressed) => {
                            if cfg!(target_os = "macos") {
                                if !state.modifiers.state().super_key() {
                                    return;
                                }
                            } else {
                                if !state.modifiers.state().control_key() {
                                    return;
                                }
                            }

                            state.tab_datas.remove(state.active_tab);

                            if state.active_tab >= state.tab_datas.len() && state.active_tab > 0 {
                                state.active_tab -= 1;
                            }

                            if state.tab_datas.len() == 0 {
                                state.tab_datas.push(TabData::empty_from(NEW_TAB_URL));
                                state.active_tab = 0;
                            }

                            if let Some(callbacks) = &self.callbacks {
                                let tab_data = &state.tab_datas[state.active_tab];
                                (callbacks.open_tab)(tab_data);
                            }
                        }
                        _ => {}
                    }
                } else {
                    match (code, key_state) {
                        (KeyCode::Escape, ElementState::Pressed) => {
                            state.address_bar_active = false;
                        }
                        (KeyCode::Enter, ElementState::Pressed) => {
                            let url = state.address_bar_input.clone();
                            state.address_bar_active = false;

                            if let Some(callbacks) = &self.callbacks {
                                (callbacks.link_callback)(&url);
                            }
                        }
                        (KeyCode::Backspace, ElementState::Pressed) => {
                            state.address_bar_input.pop();
                        }
                        _ => match logical_key {
                            Key::Character(c) => {
                                if key_state == ElementState::Pressed {
                                    state.address_bar_input.push_str(&c);
                                }
                            }
                            Key::Named(NamedKey::Space) => {
                                if key_state == ElementState::Pressed {
                                    state.address_bar_input.push(' ');
                                }
                            }
                            _ => {}
                        },
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::OpenUrl(url) => {
                if self.agent.is_none() || self.agent.is_none() {
                    return;
                }

                let state = self.state.as_mut().unwrap();
                let agent = self.agent.as_ref().unwrap();

                let tab_data = state.tab_datas.get_mut(state.active_tab).unwrap();

                if tab_data.url != url {
                    tab_data.prev_urls.push(tab_data.url.clone());

                    tab_data.url = url.clone();
                    tab_data.scroll_x = 0.0;
                    tab_data.scroll_y = 0.0;
                    tab_data.document = None;
                    tab_data.layout = None;
                }

                self.document = Some(tab_data.document.clone().unwrap_or_else(|| {
                    let doc = agent.borrow_mut().open(&url).unwrap();
                    tab_data.document = Some(Rc::clone(&doc));
                    doc
                }));

                let layout = tab_data.layout.clone().unwrap_or_else(|| {
                    let layout = Layout::make_layout(
                        self.document.as_ref().unwrap().clone(),
                        (
                            state.window.inner_size().width as f64,
                            state.window.inner_size().height as f64,
                        ),
                    );

                    let rc = Rc::new(RefCell::new(layout));

                    tab_data.layout = Some(rc.clone());
                    rc
                });

                state.viewport_height = layout
                    .borrow()
                    .root_box
                    .as_ref()
                    .unwrap()
                    .borrow()
                    ._content_height;
                state.layout = Some(layout.borrow().clone());

                state.address_bar_input = url;
                state.address_bar_active = false;
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
        let link_proxy = event_loop.create_proxy();
        let open_tab_proxy = event_loop.create_proxy();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        self.callbacks = Some(CallbackData {
            link_callback: Box::new(move |url: &str| {
                let _ = link_proxy.send_event(AppEvent::OpenUrl(url.to_string()));
            }),
            open_tab: Box::new(move |tab_data: &TabData| {
                let _ = open_tab_proxy.send_event(AppEvent::OpenUrl(tab_data.url.clone()));
            }),
        });

        let _ = event_loop.run_app(self);
    }
}
