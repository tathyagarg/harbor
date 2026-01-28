use std::{
    cell::RefCell,
    fmt::Debug,
    ops::{Add, Deref},
    rc::{Rc, Weak},
};

use crate::{
    css::{
        colors::Color,
        cssom::{CSSDeclaration, ComputedStyle},
        properties::{
            Background, CSSParseable, Display, Font, FontFamily, FontSize, FontStyle, FontWeight,
            Image, LineHeight, Margin, MarginValue, Origin, Position, PositionValue, RepeatStyle,
            WidthValue,
        },
    },
    globals::{DEFAULT_FONT_FAMILY, FONTS},
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

    pub fn top(&self) -> f64 {
        self.0
    }

    pub fn right(&self) -> f64 {
        self.1
    }

    pub fn bottom(&self) -> f64 {
        self.2
    }

    pub fn left(&self) -> f64 {
        self.3
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

    pub fn is_none(&self) -> bool {
        self.0 == 0.0 && self.1 == 0.0 && self.2 == 0.0 && self.3 == 0.0
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

    Block,

    Inline,

    ListItem,
    Marker,

    None,
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
    pub _content_width: f64,

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

    pub associated_node: Option<Rc<RefCell<NodeKind>>>,
    // pub associated_style: ComputedStyle,
}

impl Debug for Box {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Box");

        let mut res = s
            .field("content_width", &self._content_width)
            .field("content_height", &self._content_height)
            // .field("padding", &self._padding)
            // .field("border", &self._border)
            // .field("margin", &self._margin)
            .field("box_type", &self._box_type)
            .field("position_x", &self._position_x.unwrap_or(0.0))
            .field("position_y", &self._position_y.unwrap_or(0.0))
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
            );

        if let Some(node_rc) = &self.associated_node {
            if let NodeKind::Element(element_rc) = node_rc.borrow().deref() {
                let element = element_rc.borrow();
                res = res.field("associated_style", &element.style());
            }
        }

        res.finish()
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

    pub fn get_font_size(&self) -> f64 {
        if let Some(node_rc) = &self.associated_node {
            if let Some(style) = node_rc.borrow().style() {
                style.font.resolved_font_size();
            }
        }

        16.0
    }

    pub fn get_line_height(&self) -> f64 {
        if let Some(node_rc) = &self.associated_node {
            if let Some(style) = node_rc.borrow().style() {
                return style.font.resolved_line_height().unwrap_or(19.2);
            }
        }

        19.2
    }

    pub fn build_doc_box_tree(
        doc: &Rc<RefCell<Document>>,
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

            associated_node: None,
        };

        let doc_borrowed = doc.borrow();
        let doc_node = doc_borrowed._node.borrow();
        let first_element = doc_node
            .child_nodes()
            .iter()
            .find(|node_rc| matches!(node_rc.borrow().deref(), NodeKind::Element(_)));

        let tree = root_box.build_box_tree(first_element.unwrap(), &mut vec![]);

        tree
    }

    pub fn get_hovered_elems(
        root: &Rc<RefCell<Box>>,
        pos_x: f64,
        pos_y: f64,
        parent_x: f64,
        parent_y: f64,
    ) -> Vec<Rc<RefCell<Element>>> {
        let mut hovered_elems = Vec::new();

        let box_borrowed = root.borrow();

        let box_x = parent_x + box_borrowed._position_x.unwrap_or(0.0);
        let box_y = parent_y + box_borrowed._position_y.unwrap_or(0.0);

        let box_width = box_borrowed._content_width;
        let box_height = box_borrowed._content_height;

        if pos_x >= box_x
            && pos_x <= box_x + box_width
            && pos_y >= box_y
            && pos_y <= box_y + box_height
        {
            if let Some(node_rc) = &box_borrowed.associated_node {
                if let NodeKind::Element(element_rc) = node_rc.borrow().deref() {
                    hovered_elems.push(Rc::clone(element_rc));
                }
            }

            for child in box_borrowed.children.iter() {
                let mut child_hovered =
                    Box::get_hovered_elems(&Rc::clone(child), pos_x, pos_y, box_x, box_y);
                hovered_elems.append(&mut child_hovered);
            }
        }

        hovered_elems
    }

    pub fn build_box_tree(
        &mut self,
        tree: &Rc<RefCell<NodeKind>>,
        parents: &mut Vec<Weak<RefCell<Box>>>,
    ) -> Option<Rc<RefCell<Box>>> {
        match tree.borrow().deref() {
            NodeKind::Element(element_rc) if element_rc.borrow().local_name.as_str() != "head" => {
                let element = element_rc.borrow();

                // let display = match element.local_name.as_str() {
                //     "span" | "em" | "strong" => BoxType::Inline,
                //     _ => BoxType::Block,
                // };

                let parent_box = Rc::new(RefCell::new(Box {
                    _content_width: 0.0,
                    _content_height: 0.0,
                    _padding: Edges::empty(),
                    _border: Edges::empty(),
                    _margin: element.style().margin.to_edges(parents),
                    _box_type: element.style().display.to_box_type(),
                    _position_x: None,
                    _position_y: None,
                    children: vec![],

                    associated_node: Some(Rc::clone(tree)),
                }));
                parents.push(Rc::downgrade(&parent_box));

                let this_box = if element.style().display == Display::ListItem {
                    parent_box.borrow_mut().children = vec![
                        Rc::new(RefCell::new(Box {
                            _content_width: 0.0,
                            _content_height: 0.0,
                            _padding: Edges::empty(),
                            _border: Edges::empty(),
                            _margin: Edges::empty(),
                            _box_type: BoxType::Marker,
                            _position_x: None,
                            _position_y: None,
                            children: vec![],

                            associated_node: None,
                        })),
                        Rc::new(RefCell::new(Box {
                            _content_width: 0.0,
                            _content_height: 0.0,
                            _padding: Edges::empty(),
                            _border: Edges::empty(),
                            _margin: element.style().margin.to_edges(parents),
                            _box_type: BoxType::Block,
                            _position_x: None,
                            _position_y: None,
                            children: vec![],

                            associated_node: Some(Rc::clone(tree)),
                        })),
                    ];

                    parent_box.borrow_mut().associated_node = None;

                    let content_box = parent_box.borrow().children[1].clone();
                    parents.push(Rc::downgrade(&content_box));
                    content_box
                } else {
                    parent_box.clone()
                };

                for child in element._node.borrow().child_nodes().iter() {
                    if let Some(child_box) = self.build_box_tree(&child, parents) {
                        this_box.borrow_mut().children.push(child_box);
                    }
                }

                parents.pop();
                if element.style().display == Display::ListItem {
                    parents.pop();
                }

                Some(parent_box)
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

                    associated_node: Some(Rc::clone(tree)),
                }));

                Some(text_box)
            }
            _ => None,
        }
    }

    /// Container width is required for width resolution
    /// Container height currently serves no purpose but maybe it could be used in the future for
    /// height resolution
    pub fn layout(
        &mut self,
        container_width: Option<f64>,
        container_height: Option<f64>,
        first_child: bool,
        last_child: bool,
        parents: &mut Vec<Rc<RefCell<Element>>>,
    ) -> (f64, f64, bool) {
        match self._box_type {
            BoxType::Block => self.layout_block(container_width, container_height, parents),
            BoxType::Inline => self.layout_inline(
                container_width,
                container_height,
                first_child,
                last_child,
                parents,
            ),
            BoxType::ListItem => {
                let marker_box_rc = self.children.get(0).unwrap().clone();
                let content_box_rc = self.children.get(1).unwrap().clone();

                let mut marker_box = marker_box_rc.borrow_mut();
                let (marker_width, marker_height, _) = marker_box.layout(
                    container_width,
                    container_height,
                    first_child,
                    last_child,
                    parents,
                );

                let mut content_box = content_box_rc.borrow_mut();

                let (content_width, content_height, _) = content_box.layout(
                    container_width,
                    container_height,
                    first_child,
                    last_child,
                    parents,
                );

                content_box._position_x = Some(content_box._margin.left());
                content_box._position_y = Some(content_box._margin.top());

                self._content_width =
                    marker_width + content_width + content_box._margin.horizontal();
                self._content_height = marker_height.max(content_height);

                (self._content_width, self._content_height, true)
            }
            BoxType::Marker => {
                self._content_width = self.get_font_size();
                self._content_height = self.get_font_size();

                self._position_x = Some(self._content_width / 2.0);
                self._position_y = Some(self.get_line_height() * 0.5);

                (self._content_width, self._content_height, false)
            }
            BoxType::None => (0.0, 0.0, false),
            _ => {
                todo!("Layout for box type: {:?}", self._box_type);
            }
        }
    }

    pub fn style(&self) -> Option<ComputedStyle> {
        if let Some(node_rc) = &self.associated_node {
            if let NodeKind::Element(element_rc) = node_rc.borrow().deref() {
                let element = element_rc.borrow();
                return Some(element.style().clone());
            }
        }

        None
    }

    pub fn layout_block(
        &mut self,
        container_width: Option<f64>,
        container_height: Option<f64>,
        parents: &mut Vec<Rc<RefCell<Element>>>,
    ) -> (f64, f64, bool) {
        if let Some(node_rc) = &self.associated_node {
            if let NodeKind::Element(element_rc) = node_rc.borrow().deref() {
                parents.push(element_rc.clone());
            }
        }

        let initial_x = self._margin.left() + self._border.3 + self._padding.3;
        let initial_y = self._margin.top() + self._border.0 + self._padding.0;

        let mut cursor_x = initial_x;
        let mut cursor_y = initial_y;

        let mut inline_run: Vec<(Rc<RefCell<Box>>, bool, bool)> = Vec::new();

        let flush_inline_run =
            |run: &mut Vec<(Rc<RefCell<Box>>, bool, bool)>,
             cursor_x: &mut f64,
             cursor_y: &mut f64,
             content_width: &mut f64,
             parents: &mut Vec<Rc<RefCell<Element>>>| {
                if run.is_empty() {
                    return;
                }

                let mut line_width = 0.0;
                let mut line_height: f64 = 0.0;

                for (child_rc, first, last) in run.drain(..) {
                    let mut child = child_rc.borrow_mut();

                    child._position_x = Some(*cursor_x - initial_x + line_width);
                    child._position_y = Some(*cursor_y - initial_y);

                    let (w, h, go_to_next_line) =
                        child.layout(container_width, container_height, first, last, parents);

                    line_width += w + child._margin.horizontal();
                    line_height = line_height.max(h + child._margin.vertical());

                    if go_to_next_line {
                        *cursor_y += line_height;
                        *cursor_x = initial_x;
                        *content_width = content_width.max(line_width);

                        line_width = 0.0;
                    }
                }

                *cursor_y += line_height;
                *cursor_x = initial_x;
                *content_width = content_width.max(line_width);
            };

        let mut prev_child: Option<Rc<RefCell<Box>>> = None;
        for (i, child_box_rc) in self.children.iter().enumerate() {
            let child_box_type = child_box_rc.borrow()._box_type.clone();

            match child_box_type {
                BoxType::Inline => {
                    // prev_child = None;
                    inline_run.push((child_box_rc.clone(), i == 0, i == self.children.len() - 1));
                }
                BoxType::Block => {
                    flush_inline_run(
                        &mut inline_run,
                        &mut cursor_x,
                        &mut cursor_y,
                        &mut self._content_width,
                        parents,
                    );

                    let mut child = child_box_rc.borrow_mut();

                    if let Some(prev_child_rc) = &prev_child {
                        let prev = prev_child_rc.borrow();

                        if matches!(prev._box_type, BoxType::Block)
                            && prev._padding.is_none()
                            && child._padding.is_none()
                            && prev._border.is_none()
                            && child._border.is_none()
                        {
                            if child._margin.top() > prev._margin.bottom() {
                                cursor_y -= prev._margin.bottom();
                            }
                        }
                    }

                    child._position_x = Some(cursor_x);
                    child._position_y = Some(cursor_y);

                    let (w, h, go_to_next_line) = child.layout(
                        container_width,
                        container_height,
                        i == 0,
                        i == self.children.len() - 1,
                        parents,
                    );

                    cursor_y += h + child._margin.bottom();
                    if go_to_next_line {
                        cursor_x = initial_x;
                        cursor_y += child.get_line_height();
                    }

                    self._content_width = self._content_width.max(w + child._margin.horizontal());
                    prev_child = Some(child_box_rc.clone());
                }
                _ => {
                    let mut child = child_box_rc.borrow_mut();

                    child._position_x = Some(cursor_x);
                    child._position_y = Some(cursor_y);

                    let (w, h, go_to_next_line) = child.layout(
                        container_width,
                        container_height,
                        i == 0,
                        i == self.children.len() - 1,
                        parents,
                    );

                    cursor_y += h + child._margin.bottom();
                    if go_to_next_line {
                        cursor_x = initial_x;
                        cursor_y += child.get_line_height();
                    }

                    self._content_width = self._content_width.max(w + child._margin.horizontal());
                }
            }
        }

        // flush trailing inline content
        flush_inline_run(
            &mut inline_run,
            &mut cursor_x,
            &mut cursor_y,
            &mut self._content_width,
            parents,
        );

        self._content_height = cursor_y;

        if !matches!(self.style().unwrap().width, WidthValue::Auto) {
            if let Some(node_rc) = &self.associated_node {
                if let NodeKind::Element(element_rc) = node_rc.borrow().deref() {
                    let element = element_rc.borrow();
                    self._content_width = element
                        .style()
                        .width
                        .resolve(container_width.unwrap_or(0.0));
                }
            }
        }

        if let Some(node_rc) = &self.associated_node {
            if let NodeKind::Element(_) = node_rc.borrow().deref() {
                parents.pop();
            }
        }

        (self._content_width, self._content_height, false)
    }

    /// Layout for inline boxes
    /// Returns (total_width, total_height, go_to_next_line)
    pub fn layout_inline(
        &mut self,
        _container_width: Option<f64>,
        _container_height: Option<f64>,
        first_child: bool,
        last_child: bool,
        parents: &mut Vec<Rc<RefCell<Element>>>,
    ) -> (f64, f64, bool) {
        let mut pen_x = 0.0;
        let mut pen_y = 0.0;

        let node = self.associated_node.as_ref().unwrap().borrow().clone();

        match node {
            NodeKind::Text(text_node_rc) => {
                if text_node_rc.borrow().data().trim().is_empty() {
                    // TODO: Handle pre
                    return (0.0, 0.0, false);
                }

                let parent_borrow = parents.last().unwrap().borrow();
                let style = parent_borrow.style();

                let family = style.font.family();
                let weight = style.font.resolved_font_weight().unwrap_or(400) as u16;

                let mut iterator = family.entries.iter();

                let ttc = loop {
                    let entry = iterator.next();
                    if let Some(entry) = entry {
                        if let Some(f) = FONTS.get(&entry.value()) {
                            break Some(f);
                        }
                    } else {
                        break FONTS.get(DEFAULT_FONT_FAMILY);
                    }
                };

                let font = if matches!(style.font.style(), FontStyle::Italic) {
                    ttc.and_then(|ttc| ttc.get_italic_font_by_weight(weight))
                        .or(ttc.and_then(|ttc| ttc.get_font_by_weight(weight)))
                } else {
                    ttc.and_then(|ttc| ttc.get_font_by_weight(weight))
                }
                .or(ttc.and_then(|ttc| ttc.get_regular_font()));

                let scale = style.font.resolved_font_size().unwrap_or(16.0)
                    / font.unwrap().units_per_em() as f64;

                let mut new_data = String::new();

                let chars = {
                    let text_node = text_node_rc.borrow();
                    let data = text_node.data();
                    if first_child && last_child {
                        data.trim().chars().collect::<Vec<char>>()
                    } else if first_child {
                        data.trim_start().chars().collect::<Vec<char>>()
                    } else if last_child {
                        data.trim_end().chars().collect::<Vec<char>>()
                    } else {
                        data.chars().collect::<Vec<char>>()
                    }
                };

                let mut last_was_space = false;

                for ch in chars {
                    if ch != '\n' && ch != '\r' && ch != '\t' {
                        if last_was_space && ch == ' ' {
                            continue;
                        }

                        last_was_space = ch == ' ';
                        new_data.push(ch);

                        let aw = font
                            .and_then(|font| {
                                font.advance_width(
                                    font.glyph_index(ch as u32)
                                        .unwrap_or_else(|| font.last_glyph_index().unwrap()),
                                )
                                // .map(|aw| aw as f64 * self._font_size.unwrap_or(16.0))
                                .map(|aw| aw as f64 * scale)
                            })
                            .unwrap_or_else(|| {
                                font.and_then(|font| {
                                    font.rawdog_advance_width(
                                        font.glyph_index(ch as u32)
                                            .unwrap_or_else(|| font.last_glyph_index().unwrap()),
                                    )
                                })
                                .map(|aw| aw as f64 * scale)
                                .unwrap_or(0.0)
                            });

                        pen_x += aw;
                    } else {
                        // TODO: handle pre
                    }
                }

                text_node_rc.borrow_mut().set_data(&new_data);
                self._content_height = self
                    ._content_height
                    .max(style.font.resolved_line_height().unwrap_or(19.2));

                self._content_width = self._content_width.max(pen_x);
            }
            NodeKind::Element(e) => {
                if e.borrow().local_name.as_str() == "br" {
                    pen_x = 0.0;
                    self._content_height = self._content_height.max(
                        e.borrow()
                            .style()
                            .font
                            .resolved_line_height()
                            .unwrap_or(19.2),
                    );
                    return (pen_x, self._content_height, true);
                }

                parents.push(e);

                for (i, child_box) in self.children.iter().enumerate() {
                    let mut child_box = child_box.borrow_mut();

                    child_box._position_x = Some(pen_x);
                    child_box._position_y = Some(pen_y);

                    let (advance, line_height, go_to_next_line) = child_box.layout_inline(
                        None,
                        None,
                        i == 0,
                        i == self.children.len() - 1,
                        parents,
                    );

                    pen_x += advance;
                    self._content_height = self._content_height.max(line_height);

                    if go_to_next_line {
                        self._content_width = self._content_width.max(pen_x);
                        pen_x = 0.0;
                        pen_y += line_height;
                    }
                }

                self._content_width = self._content_width.max(pen_x);

                parents.pop();
            }
            _ => {}
        }

        (self._content_width, self._content_height, false)
    }
}

fn compute_doc_styles(doc: &Rc<RefCell<Document>>) {
    let doc_borrow = doc.borrow();
    let doc_node = doc_borrow._node.borrow();
    let children = doc_node.child_nodes();

    for node_rc in children.iter() {
        let node = node_rc.borrow();
        if let NodeKind::Element(element_rc) = node.deref() {
            let mut element = element_rc.borrow_mut();
            element.compute_element_styles(None);
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

fn handle_background_property(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "background-color" => {
            let color = Color::from_cv(&mut stream);
            if let Some(color) = color {
                style.background.set_color(color);
            }
        }
        "background-image" => {
            let bg_image = Image::parse_multiple_images(&mut stream);
            style.background.set_images(bg_image);
        }
        "background-repeat" => {
            let repeat = RepeatStyle::parse_multiple_repeat_styles(&mut stream);
            style.background.set_repeat_styles(repeat);
        }
        "background-position" => {
            let position = PositionValue::parse_multiple_positions(&mut stream);
            style.background.set_positions(position);
        }
        "background-origin" => {
            let origin = Origin::parse_multiple_origins(&mut stream);
            style.background.set_origins(origin);
        }
        _ => {}
    }
}

fn handle_font(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
) {
    let mut stream = InputStream::new(&declaration.value);

    let font = Font::from_cv(&mut stream);
    if let Some(font) = font {
        style.font = font;
        style.font.resolve_font_size(parents.unwrap_or(&vec![]));
        style.font.resolve_font_weight(parents.unwrap_or(&vec![]));
    }
}

fn handle_font_property(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "font-family" => {
            let family = FontFamily::from_cv(&mut stream);
            if let Some(family) = family {
                style.font.set_family(family);
            }
        }
        "font-size" => {
            let size = FontSize::from_cv(&mut stream);
            if let Some(size) = size {
                style.font.set_size(size);
                style.font.resolve_font_size(parents.unwrap_or(&vec![]));
            }
        }
        "font-weight" => {
            let weight = FontWeight::from_cv(&mut stream);
            if let Some(weight) = weight {
                style.font.set_weight(weight);
                style.font.resolve_font_weight(parents.unwrap_or(&vec![]));
            }
        }
        "line-height" => {
            let line_height = LineHeight::from_cv(&mut stream);
            if let Some(line_height) = line_height {
                style.font.set_line_height(line_height);
            }
        }
        "font-style" => {
            let font_style = FontStyle::from_cv(&mut stream);
            if let Some(font_style) = font_style {
                style.font.set_style(font_style);
            }
        }
        _ => {}
    }
}

fn handle_margin(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    let mut stream = InputStream::new(&declaration.value);

    let margin = Margin::from_cv(&mut stream);
    if let Some(margin) = margin {
        style.margin = margin;
    }
}

fn handle_margin_property(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "margin-top" => {
            let top = MarginValue::from_cv(&mut stream);
            if let Some(top) = top {
                style.margin.top = top;
            }
        }
        "margin-right" => {
            let right = MarginValue::from_cv(&mut stream);
            if let Some(right) = right {
                style.margin.right = right;
            }
        }
        "margin-bottom" => {
            let bottom = MarginValue::from_cv(&mut stream);
            if let Some(bottom) = bottom {
                style.margin.bottom = bottom;
            }
        }
        "margin-left" => {
            let left = MarginValue::from_cv(&mut stream);
            if let Some(left) = left {
                style.margin.left = left;
            }
        }
        _ => {}
    }
}

pub fn handle_declaration(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
) {
    match declaration.property_name.as_str() {
        "color" => {
            let mut stream = InputStream::new(&declaration.value);
            style.color = Color::from_cv(&mut stream).unwrap_or(Color::default());
        }
        "background" => {
            handle_background(declaration, style);
        }
        prop if prop.starts_with("background-") => {
            handle_background_property(declaration, style);
        }
        "font" => {
            handle_font(declaration, style, parents);
        }
        prop if prop.starts_with("font-") || prop == "line-height" => {
            handle_font_property(declaration, style, parents);
        }
        "width" => {
            let mut stream = InputStream::new(&declaration.value);
            style.width = WidthValue::from_cv(&mut stream).unwrap_or_default();
        }
        "display" => {
            let mut stream = InputStream::new(&declaration.value);
            style.display = Display::from_cv(&mut stream).unwrap_or_default();
        }
        "margin" => {
            handle_margin(declaration, style);
        }
        prop if prop.starts_with("margin-") => {
            handle_margin_property(declaration, style);
        }
        "position" => {
            let mut stream = InputStream::new(&declaration.value);
            style.position = Position::from_cv(&mut stream).unwrap_or_default();
        }
        _ => {
            // todo!(
            //     "Implement handling for property: {}",
            //     declaration.property_name
            // );
        }
    }
}
