use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::render::{RendererIdentifier, TextRenderer};

use crate::globals::FONTS;

use crate::css::r#box;
use crate::html5::dom::Document;

#[derive(Clone)]
pub struct Layout {
    pub document: Rc<RefCell<Document>>,
    pub root_box: Option<Rc<RefCell<r#box::Box>>>,

    pub _renderers: HashMap<RendererIdentifier, Option<TextRenderer>>,

    _window_size: (f64, f64),
}

impl Layout {
    pub fn new(document: Rc<RefCell<Document>>, window_size: (f64, f64)) -> Self {
        let mut this = Layout {
            document,
            root_box: None,
            _renderers: HashMap::new(),
            _window_size: window_size,
        };

        this.populate_renderers();

        this
    }

    pub fn make_tree(&mut self) {
        let root_box = r#box::Box::build_doc_box_tree(&self.document, self._window_size);
        self.root_box = root_box;
    }

    pub fn layout(&mut self) {
        if let Some(root_box) = &self.root_box {
            root_box.borrow_mut().layout(
                Some(self._window_size.0),
                Some(self._window_size.1),
                // fuhhh
                false,
                false,
                &mut vec![],
                &self._renderers,
            );
        }

        // for (_, renderer) in self._renderers.iter_mut() {
        //     if let Some(r) = renderer {
        //         r.resized((self._window_size.0 as f32, self._window_size.1 as f32));
        //     }
        // }
    }

    pub fn get_renderer(&self, name: String) -> Option<&TextRenderer> {
        for (identifier, renderer_option) in self._renderers.iter() {
            if identifier.font_family == name {
                if let Some(renderer) = renderer_option {
                    return Some(renderer);
                }
            }
        }

        None
    }

    pub fn resized(&mut self, new_size: (f64, f64)) {
        self._window_size = new_size;
        self.layout();
    }

    pub fn populate_renderers(&mut self) {
        for (font_name, font_collection) in FONTS.iter() {
            for font in &font_collection.table_directories {
                let identifier = RendererIdentifier {
                    font_family: font_name.clone(),
                    font_weight: font.get_weight().unwrap_or(400),
                    italic: font.is_italic(),
                };

                let renderer = TextRenderer {
                    _associated_italic: identifier.italic,
                    _associated_weight: identifier.font_weight,
                    font: font.clone(),
                    glyph_cache: HashMap::new(),
                };

                self._renderers.insert(identifier, Some(renderer));
            }
        }

        // for (font_name, font) in FONTS.iter() {
        //     let renderer = TextRenderer {
        //         font: Some(font.clone()),
        //     };
        //     self._renderers.insert(font_name.clone(), Some(renderer));
        // }
    }
}
