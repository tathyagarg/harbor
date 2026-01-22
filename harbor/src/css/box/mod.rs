#![allow(dead_code)]

use std::{
    cell::RefCell,
    fmt::Debug,
    ops::{Add, Deref},
    rc::{Rc, Weak},
};

use crate::{
    css::{
        colors::Color,
        cssom::{
            CSSDeclaration, CSSRuleNode, CSSRuleType, CSSStyleRuleData, CSSStyleSheetExt,
            ComputedStyle,
        },
        parser::ComponentValue,
        properties::{Background, CSSParseable, WidthValue},
        selectors::MatchesElement,
        tokenize::CSSToken,
    },
    globals::FONTS,
    html5::dom::{Document, Element, NodeKind},
    infra::InputStream,
};

/// Represents the edges of a box: top, right, bottom, left
#[derive(Debug, Clone, Copy)]
pub struct Edges(pub f64, pub f64, pub f64, pub f64);

impl Edges {
    pub fn empty() -> Self {
        Edges(0.0, 0.0, 0.0, 0.0)
    }

    /// Calculate total horizontal size (left + right)
    pub fn horizontal(&self) -> f64 {
        self.3 + self.1
    }

    /// Calculate total vertical size (top + bottom)
    pub fn vertical(&self) -> f64 {
        self.0 + self.2
    }

    pub fn update(&mut self, top: f64, right: f64, bottom: f64, left: f64) {
        self.0 = top;
        self.1 = right;
        self.2 = bottom;
        self.3 = left;
    }

    pub fn update_top(&mut self, top: f64) {
        self.0 = top;
    }

    pub fn update_right(&mut self, right: f64) {
        self.1 = right;
    }

    pub fn update_bottom(&mut self, bottom: f64) {
        self.2 = bottom;
    }

    pub fn update_left(&mut self, left: f64) {
        self.3 = left;
    }
}

impl Add<Edges> for Edges {
    type Output = Edges;

    fn add(self, other: Edges) -> Edges {
        Edges(
            self.0 + other.0,
            self.1 + other.1,
            self.2 + other.2,
            self.3 + other.3,
        )
    }
}

/// A box's type affects, in part, its behavior in the visual formatting model. The 'display' property ... specifies a box's type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    /// Created by :root
    Initial,

    /// display: block | list-item | table;
    Block,

    Inline,
}

pub enum FormattingContext {
    BlockFormattingContext,
    InlineFormattingContext,
}

/// The CSS box model describes the rectangular boxes that are generated for elements in the document tree and laid out according to the visual formatting model.
///
/// Each box has a content area (e.g., text, an image, etc.) and optional surrounding padding, border, and margin areas;
///
/// content edge or inner edge
///     The content edge surrounds the rectangle given by the width and height of the box, which often depend on the element's rendered content. The four content edges define the box's content box.
/// padding edge
///     The padding edge surrounds the box padding. If the padding has 0 width, the padding edge is the same as the content edge. The four padding edges define the box's padding box.
/// border edge
///     The border edge surrounds the box's border. If the border has 0 width, the border edge is the same as the padding edge. The four border edges define the box's border box.
/// margin edge or outer edge
///     The margin edge surrounds the box margin. If the margin has 0 width, the margin edge is the same as the border edge. The four margin edges define the box's margin box.
#[derive(Clone)]
pub struct Box {
    /// Width of the content area
    _content_width: f64,

    /// Height of the content area
    _content_height: f64,

    /// Padding edges: top, right, bottom, left
    _padding: Edges,

    /// Border edges: top, right, bottom, left
    _border: Edges,

    /// Margin edges: top, right, bottom, left
    _margin: Edges,

    /// The box type (block or inline)
    pub _box_type: BoxType,

    /// The X position of the box, set during layout formation
    _position_x: Option<f64>,

    /// The Y position of the box, set during layout formation
    _position_y: Option<f64>,

    pub children: Vec<Rc<RefCell<Box>>>,

    /* Inline Formatting Context Specific Properties */
    /// TODO: Switch to using a proper Font struct
    pub _font_family: Option<String>,
    pub _font_size: Option<f64>,
    pub _font_weight: Option<String>,
    pub _line_height: Option<f64>,

    pub associated_node: Option<Rc<RefCell<NodeKind>>>,

    pub associated_style: ComputedStyle,
}

impl Debug for Box {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Box")
            .field("content_width", &self._content_width)
            .field("content_height", &self._content_height)
            // .field("padding", &self._padding)
            // .field("border", &self._border)
            // .field("margin", &self._margin)
            .field("box_type", &self._box_type)
            .field("position_x", &self._position_x.unwrap_or(-67.0))
            .field("position_y", &self._position_y.unwrap_or(-67.0))
            .field("children_count", &self.children.len())
            .field("children", &self.children)
            .field(
                "associated_node",
                &self
                    .associated_node
                    .as_ref()
                    .and_then(|rc| {
                        let borrowed = rc.borrow();
                        match borrowed.deref() {
                            NodeKind::Element(node) => Some(node.borrow().local_name.clone()),
                            NodeKind::Text(node) => Some(format!("Text: {}", node.borrow().data())),
                            _ => None,
                        }
                    })
                    .unwrap_or_else(|| "None".to_string()),
            )
            .field("associated_style", &self.associated_style)
            .finish()
    }
}

impl Box {
    /* Content Edges */
    pub fn content_edges(&self) -> Edges {
        Edges(0.0, self._content_width, self._content_height, 0.0)
    }

    pub fn content_area(&self) -> f64 {
        self._content_width * self._content_height
    }

    pub fn update_content_size(&mut self, width: f64, height: f64) {
        self._content_width = width;
        self._content_height = height;
    }

    pub fn update_content_width(&mut self, width: f64) {
        self._content_width = width;
    }

    pub fn update_content_height(&mut self, height: f64) {
        self._content_height = height;
    }

    /* Padding Edges */
    pub fn padding_edges(&self) -> Edges {
        let content = self.content_edges();
        content + self._padding
    }

    pub fn padding(&self) -> &Edges {
        &self._padding
    }

    pub fn padding_mut(&mut self) -> &mut Edges {
        &mut self._padding
    }

    /* Border Edges */
    pub fn border_edges(&self) -> Edges {
        let padding = self.padding_edges();
        padding + self._border
    }

    pub fn border(&self) -> &Edges {
        &self._border
    }

    pub fn border_mut(&mut self) -> &mut Edges {
        &mut self._border
    }

    /* Margin Edges */
    pub fn margin_edges(&self) -> Edges {
        let border = self.border_edges();
        border + self._margin
    }

    pub fn margin(&self) -> &Edges {
        &self._margin
    }

    pub fn margin_mut(&mut self) -> &mut Edges {
        &mut self._margin
    }

    pub fn position(&self) -> (f64, f64) {
        (
            self._position_x.unwrap_or(0.0),
            self._position_y.unwrap_or(0.0),
        )
    }

    pub fn build_doc_box_tree(
        doc: &mut Document,
        window_size: (f64, f64),
    ) -> Option<Rc<RefCell<Box>>> {
        compute_doc_styles(doc);

        // For now, create a simple box for the document root
        let mut root_box = Box {
            _content_width: window_size.0,
            _content_height: window_size.1,
            _padding: Edges::empty(),
            _border: Edges::empty(),
            _margin: Edges::empty(),
            _box_type: BoxType::Block,
            _position_x: Some(0.0),
            _position_y: Some(0.0),
            children: vec![],

            _font_family: None,
            _font_size: None,
            _font_weight: None,
            _line_height: None,

            associated_node: None,

            associated_style: ComputedStyle::default(),
        };

        let doc_borrowed = doc._node.borrow();
        let first_element = doc_borrowed
            .child_nodes()
            .iter()
            .find(|node_rc| matches!(node_rc.borrow().deref(), NodeKind::Element(_)));

        let tree = root_box.build_box_tree(first_element.unwrap(), None);

        tree
    }

    pub fn build_box_tree(
        &mut self,
        tree: &Rc<RefCell<NodeKind>>,
        parent: Option<&Weak<RefCell<Box>>>,
    ) -> Option<Rc<RefCell<Box>>> {
        match tree.borrow().deref() {
            NodeKind::Element(element_rc) if element_rc.borrow().local_name.as_str() != "head" => {
                let element = element_rc.borrow();

                let display = match element.local_name.as_str() {
                    "span" | "em" | "strong" => BoxType::Inline,
                    _ => BoxType::Block,
                };

                let mag = 7.5;

                let font_size = match element.local_name.as_str() {
                    "h1" => 32.0,
                    "h2" => 24.0,
                    "h3" => 18.72,
                    "h4" => 16.0,
                    "h5" => 13.28,
                    "h6" => 10.72,
                    _ => {
                        parent
                            .and_then(|weak_box| weak_box.upgrade())
                            .and_then(|parent_box_rc| parent_box_rc.borrow()._font_size)
                            .unwrap_or(16.0 * mag)
                            / mag
                    }
                } * mag;

                let line_height = font_size * 1.2;

                let this_box = Rc::new(RefCell::new(Box {
                    _content_width: 0.0,
                    _content_height: 0.0,
                    _padding: Edges::empty(),
                    _border: Edges::empty(),
                    _margin: Edges::empty(),
                    _box_type: display,
                    _position_x: None,
                    _position_y: None,
                    children: vec![],

                    _font_family: None,
                    _font_size: Some(font_size),
                    _font_weight: None,
                    _line_height: Some(line_height),

                    associated_node: Some(Rc::clone(tree)),

                    associated_style: element.style().clone(),
                }));

                for child in element._node.borrow().child_nodes().iter() {
                    if let Some(child_box) =
                        self.build_box_tree(&child, Some(&Rc::downgrade(&this_box)))
                    {
                        this_box.borrow_mut().children.push(child_box);
                    }
                }

                Some(this_box)
            }
            NodeKind::Text(_) => {
                // Text nodes can be represented as inline boxes
                let text_box = Rc::new(RefCell::new(Box {
                    _content_width: 0.0,
                    _content_height: 0.0,
                    _padding: Edges::empty(),
                    _border: Edges::empty(),
                    _margin: Edges::empty(),
                    _box_type: BoxType::Inline,
                    _position_x: None,
                    _position_y: None,
                    children: vec![],

                    _font_family: Some("Times New Roman".to_string()),
                    _font_size: Some(
                        parent
                            .and_then(|weak_box| weak_box.upgrade())
                            .and_then(|parent_box_rc| parent_box_rc.borrow()._font_size)
                            .unwrap_or(16.0),
                    ),
                    _font_weight: None,
                    _line_height: Some(
                        parent
                            .and_then(|weak_box| weak_box.upgrade())
                            .and_then(|parent_box_rc| parent_box_rc.borrow()._line_height)
                            .unwrap_or(19.2),
                    ),

                    associated_node: Some(Rc::clone(tree)),

                    associated_style: parent
                        .and_then(|weak_box| weak_box.upgrade())
                        .map_or(ComputedStyle::default(), |parent_box_rc| {
                            parent_box_rc.borrow().associated_style.clone()
                        }),
                }));

                Some(text_box)
            }
            _ => None,
        }
    }

    pub fn layout(
        &mut self,
        container_width: Option<f64>,
        container_height: Option<f64>,
        start_x: f64,
        start_y: f64,
    ) -> (f64, f64) {
        match self._box_type {
            BoxType::Block => {
                self.layout_block(container_width, container_height, start_x, start_y)
            }
            BoxType::Inline => {
                // For simplicity, treat inline boxes as block boxes in this example
                self.layout_inline(container_width, container_height, start_x, start_y)
            }
            _ => (start_x, start_y),
        }
    }

    pub fn layout_block(
        &mut self,
        container_width: Option<f64>,
        container_height: Option<f64>,
        start_x: f64,
        start_y: f64,
    ) -> (f64, f64) {
        let initial_x = start_x + self._margin.3 + self._border.3 + self._padding.3;
        let mut cursor_x = initial_x;
        let mut cursor_y = start_y;

        for child_box_rc in &self.children {
            let mut child_box = child_box_rc.borrow_mut();

            child_box._position_x = Some(0.0);
            child_box._position_y = Some(cursor_y - start_y);

            let (child_width, child_height) =
                child_box.layout(container_width, container_height, cursor_x, cursor_y);

            cursor_y += child_height + child_box._margin.vertical();
            cursor_x = initial_x;

            self._content_width = self._content_width.max(child_width);
        }

        let total_height = cursor_y - start_y;
        self._content_height = total_height;

        if !matches!(self.associated_style.width, WidthValue::Auto) {
            self._content_width = self
                .associated_style
                .width
                .resolve(container_width.unwrap_or(0.0));
        }

        (self._content_width, total_height)
    }

    pub fn layout_inline(
        &mut self,
        _container_width: Option<f64>,
        _container_height: Option<f64>,
        start_x: f64,
        start_y: f64,
    ) -> (f64, f64) {
        let mut pen_x = start_x;
        let mut pen_y = start_y;

        let node = self.associated_node.as_ref().unwrap().borrow().clone();

        match node {
            NodeKind::Text(text_node_rc) => {
                if text_node_rc.borrow().data().trim().is_empty() {
                    // TODO: Handle pre
                    return (0.0, 0.0);
                }

                let font = FONTS.get(
                    &self
                        ._font_family
                        .clone()
                        .unwrap_or_else(|| "Times New Roman".to_string()),
                );
                let scale = self._font_size.unwrap_or(16.0) / font.unwrap().units_per_em() as f64;

                let mut new_data = String::new();
                for ch in text_node_rc.borrow().data().trim().chars() {
                    if ch != '\n' && ch != '\r' && ch != '\t' {
                        new_data.push(ch);
                        let aw = font
                            .and_then(|font| {
                                font.advance_width(font.glyph_index(ch as u32).unwrap() as usize)
                                    // .map(|aw| aw as f64 * self._font_size.unwrap_or(16.0))
                                    .map(|aw| aw as f64 * scale)
                            })
                            .unwrap_or(8.0);

                        pen_x += aw;
                    } else {
                        // TODO: handle pre
                    }
                }

                text_node_rc.borrow_mut().set_data(&new_data);
                pen_y += self._line_height.unwrap_or(16.0);
            }
            NodeKind::Element(element_rc) => {
                let element = element_rc.borrow();

                for child in element._node.borrow().child_nodes().iter() {
                    let child_box_opt = self.build_box_tree(
                        child,
                        Some(&Rc::downgrade(&Rc::new(RefCell::new(self.clone())))),
                    );
                    if let Some(child_box_rc) = child_box_opt {
                        let mut child_box = child_box_rc.borrow_mut();

                        let (child_width, child_height) =
                            child_box.layout(None, None, pen_x, pen_y);

                        pen_x += child_width;
                        pen_y = pen_y.max(child_height);
                    }
                }
            }
            _ => {}
        }

        let total_width = pen_x - start_x;
        self._content_width = total_width;
        self._content_height = pen_y - start_y;

        (total_width, pen_y - start_y)
    }
}

fn compute_doc_styles(doc: &mut Document) {
    for node_rc in doc._node.borrow().child_nodes().iter() {
        let node = node_rc.borrow();
        if let NodeKind::Element(element_rc) = node.deref() {
            let mut element = element_rc.borrow_mut();
            compute_element_styles(doc, &mut element, None);
        }
    }
}

fn compute_element_styles(
    document: &Document,
    element: &mut Element,
    parents: Option<&Vec<&Element>>,
) {
    let style_sheets = document.style_sheets();

    for stylesheet in style_sheets.style_sheets.iter() {
        for rule in stylesheet.borrow().css_rules().iter() {
            match rule.deref()._type() {
                CSSRuleType::Style => {
                    let style_rule = rule
                        .deref()
                        .as_any()
                        .downcast_ref::<CSSRuleNode<CSSStyleRuleData>>()
                        .unwrap();

                    for selector in style_rule.selectors() {
                        if selector.matches(element, parents) {
                            let style = element.style_mut();

                            for declaration in style_rule.declarations() {
                                handle_declaration(declaration, style);
                            }
                        }
                    }
                }
                _ => {
                    todo!("Handle other CSS rule types");
                }
            }
        }
    }

    let mut new_parents = match parents {
        Some(p) => p.clone(),
        None => vec![],
    };
    new_parents.push(element);

    for child_rc in element._node.borrow().child_nodes().iter() {
        let child = child_rc.borrow();
        if let NodeKind::Element(child_element_rc) = child.deref() {
            let mut child_element = child_element_rc.borrow_mut();
            compute_element_styles(document, &mut child_element, Some(&new_parents));
        }
    }
}

fn handle_background(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    let mut stream = InputStream::new(&declaration.value);

    let bg = Background::from_cv(&mut stream);
    if let Some(bg) = bg {
        style.background = bg;
    }
}

fn handle_declaration(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    match declaration.property_name.as_str() {
        "color" => {
            let mut stream = InputStream::new(&declaration.value);
            style.color = Color::from_cv(&mut stream).unwrap_or(Color::default());
        }
        "background" => {
            handle_background(declaration, style);
        }
        "width" => {
            let mut stream = InputStream::new(&declaration.value);
            style.width = WidthValue::from_cv(&mut stream).unwrap_or_default();
        }
        _ => {
            // todo!(
            //     "Implement handling for property: {}",
            //     declaration.property_name
            // );
        }
    }
}
