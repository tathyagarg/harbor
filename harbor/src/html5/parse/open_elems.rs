use crate::html5;
use crate::html5::dom::{Element, IElement, INode, InsertLocation, NodeKind};
use crate::html5::parse::tree::Token;
use crate::html5::tag_groups::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct OpenElementsStack {
    pub elements: Vec<Rc<RefCell<Element>>>,
}

impl OpenElementsStack {
    pub fn new() -> OpenElementsStack {
        OpenElementsStack { elements: vec![] }
    }

    pub fn push(&mut self, element: Rc<RefCell<Element>>) {
        self.elements.push(element);
    }

    pub fn pop(&mut self) -> Option<Rc<RefCell<Element>>> {
        self.elements.pop()
    }

    pub fn pop_until(&mut self, target_name: &str) {
        while let Some(element) = self.pop() {
            if element.borrow().qualified_name() == target_name {
                break;
            }
        }
    }

    pub fn nth(&self, index: usize) -> Option<Rc<RefCell<Element>>> {
        self.elements.get(index).map(Rc::clone)
    }

    pub fn contains(&self, element: &Element) -> bool {
        let element_name = element.qualified_name();

        self.elements
            .iter()
            .any(|el| el.borrow().qualified_name() == element_name)
    }

    pub fn contains_rc(&self, element: &Rc<RefCell<Element>>) -> bool {
        let element_name = element.borrow().qualified_name();

        self.elements
            .iter()
            .any(|el| el.borrow().qualified_name() == element_name)
    }
    pub fn contains_tag(&self, tag_name: &str) -> bool {
        self.elements
            .iter()
            .any(|el| el.borrow().qualified_name() == tag_name)
    }

    /// TODO: Update to match spec
    pub fn adjusted_current_node(&self) -> Option<Rc<RefCell<Element>>> {
        // Subject to change
        self.elements.last().map(Rc::clone)
    }

    pub fn current_node(&self) -> Option<Rc<RefCell<Element>>> {
        self.elements.last().map(Rc::clone)
    }

    // fn adjusted_current_node_mut(&mut self) -> Option<&mut Weak<RefCell<Element>>> {
    //     self.elements.last_mut()
    // }

    pub fn appropriate_insertion_place(
        &mut self,
        override_target: Option<Rc<RefCell<Element>>>,
    ) -> InsertLocation {
        let target = override_target.unwrap_or_else(|| {
            self.adjusted_current_node()
                .expect("No current node for appropriate insertion place")
        });

        let adjusted_insertion_position = target.borrow().node().borrow().child_nodes().length();

        InsertLocation::new(
            Rc::new(RefCell::new(NodeKind::Element(target))),
            adjusted_insertion_position,
        )
    }

    fn insert_foreign_element(
        &mut self,
        token: &Token,
        namespace: &str,
        only_add_to_element_stack: bool,
    ) -> Rc<RefCell<Element>> {
        let mut adjusted_insertion_location = self.appropriate_insertion_place(None);

        let element = Element::from_token(
            token,
            namespace,
            &*adjusted_insertion_location.parent().borrow(),
        );

        if !only_add_to_element_stack {
            adjusted_insertion_location.insert(&mut NodeKind::Element(element.clone()));
        }

        self.push(element.clone());
        element
    }

    pub fn insert_html_element(&mut self, token: &Token) -> Rc<RefCell<Element>> {
        self.insert_foreign_element(token, html5::HTML_NAMESPACE, false)
    }

    pub fn has_element_in_specific_scope(&self, target_name: &str, scope_names: &[&str]) -> bool {
        for element in self.elements.iter().rev() {
            // Starnger Things ending was shit
            let el = element.borrow();

            if el.qualified_name() == target_name {
                return true;
            }

            if scope_names.contains(&el.qualified_name().as_str()) {
                return false;
            }
        }

        false
    }

    pub fn has_element_in_default_scope(&self, target_name: &str) -> bool {
        // TODO: Fact check list
        self.has_element_in_specific_scope(target_name, &DEFAULT_SCOPE_NAMES)
    }

    pub fn has_element_in_list_item_scope(&self, target_name: &str) -> bool {
        self.has_element_in_specific_scope(target_name, &LIST_ITEM_SCOPE_NAMES)
    }

    pub fn has_non_special_element_in_scope(&self) -> bool {
        let special_elements = [
            "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody", "td",
            "tfoot", "th", "thead", "tr", "body", "html",
        ];

        for element in self.elements.iter().rev() {
            if !special_elements.contains(&element.borrow().qualified_name().as_str()) {
                return true;
            }
        }

        false
    }

    pub fn has_element_in_button_scope(&self, target_name: &str) -> bool {
        self.has_element_in_specific_scope(target_name, &BUTTON_SCOPE_NAMES)
    }

    pub fn generate_implied_end_tags(&mut self, exclude: Option<&str>) {
        loop {
            let _current_node = match self.adjusted_current_node() {
                Some(node) => node,
                None => break,
            };

            let current_node = _current_node.borrow();

            if IMPLIED_END_TAGS.contains(&current_node.qualified_name().as_str())
                && Some(current_node.qualified_name().as_str()) != exclude
            {
                self.pop();
            } else {
                break;
            }
        }
    }

    pub fn close_p_tag(&mut self) {
        self.generate_implied_end_tags(Some("p"));
        self.pop_until("p")
    }
}
