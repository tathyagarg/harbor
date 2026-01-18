use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::font::ttf::ParsedTableDirectory;
use crate::render::TextRenderer;

use crate::globals::FONTS;

use crate::css::r#box;
use crate::html5::dom::Document;

#[derive(Clone)]
pub struct Layout {
    pub document: Rc<RefCell<Document>>,
    pub root_box: Option<Rc<RefCell<r#box::Box>>>,

    pub available_fonts: HashMap<String, ParsedTableDirectory>,
    pub _renderers: HashMap<String, Option<TextRenderer>>,

    _window_size: (f64, f64),
}

impl Layout {
    pub fn new(document: Rc<RefCell<Document>>, window_size: (f64, f64)) -> Self {
        Layout {
            document,
            root_box: None,
            available_fonts: HashMap::new(),
            _renderers: HashMap::new(),
            _window_size: window_size,
        }
    }

    pub fn make_tree(&mut self) {
        let mut document = self.document.borrow_mut();

        let root_node = document.deref_mut();
        let root_box = r#box::Box::build_doc_box_tree(root_node, self._window_size);
        self.root_box = root_box;
    }

    pub fn layout(&mut self) {
        if let Some(root_box) = &self.root_box {
            root_box.borrow_mut().layout(
                Some(self._window_size.0),
                Some(self._window_size.1),
                0.0,
                0.0,
            );
        }
    }

    pub fn register_font(&mut self, font_name: &str, font: ParsedTableDirectory) {
        self.available_fonts.insert(font_name.to_string(), font);
    }

    pub fn resized(&mut self, new_size: (f64, f64)) {
        self._window_size = new_size;
        self.layout();
    }

    pub fn populate_renderers(&mut self, window_size: (f32, f32)) {
        for (font_name, font) in FONTS.iter() {
            let renderer = TextRenderer::new()
                .with_font(font.clone())
                .with_window_size(window_size)
                .build();
            self._renderers.insert(font_name.clone(), Some(renderer));
        }
    }
}
