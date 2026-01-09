use crate::html5::dom::*;
use crate::html5::parse::Parser;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub enum ElementOrMarker {
    Element(Rc<RefCell<Element>>),
    Marker,
}

pub struct ActiveFormattingElements {
    pub elements: Vec<ElementOrMarker>,
}

impl ActiveFormattingElements {
    pub fn new() -> ActiveFormattingElements {
        ActiveFormattingElements { elements: vec![] }
    }

    pub fn last_marker(&self) -> Option<usize> {
        for (i, element) in self.elements.iter().enumerate().rev() {
            if matches!(element, ElementOrMarker::Marker) {
                return Some(i);
            }
        }
        None
    }

    pub fn push(&mut self, element: Rc<RefCell<Element>>) {
        let last_marker = self.last_marker().unwrap_or(0);
        let target_borrow = element.borrow().clone();

        if self.elements.len() >= 1
            && self.elements[last_marker + 1..]
                .iter()
                .map(|el| match el {
                    ElementOrMarker::Element(e) => e.clone(),
                    ElementOrMarker::Marker => panic!("Should not encounter marker here"),
                })
                .filter(|el| {
                    let borrowed_el = el.borrow();
                    borrowed_el.clone() == target_borrow

                    // borrowed_el.qualified_name() == element.qualified_name()
                    //     && element.attributes().len() == borrowed_el.attributes().len()
                    //     && element.attributes().iter().all(|attr| {
                    //         borrowed_el
                    //             .get_attribute(attr.local_name())
                    //             .is_some_and(|v| v == attr.value())
                    //     })
                    //     && borrowed_el.namespace_uri() == element.namespace_uri()
                })
                .count()
                >= 3
        {
            let first_matching_index = self
                .elements
                .iter()
                .position(|el| match el {
                    ElementOrMarker::Element(e) => {
                        e.borrow().clone() == target_borrow
                        // e.qualified_name() == element.qualified_name()
                        //     && element.attributes().len() == e.attributes().len()
                        //     && element.attributes().iter().all(|attr| {
                        //         e.get_attribute(attr.local_name())
                        //             .is_some_and(|v| v == attr.value())
                        //     })
                        //     && e.namespace_uri() == element.namespace_uri()
                    }
                    ElementOrMarker::Marker => false,
                })
                .unwrap();

            self.elements.remove(first_matching_index);
        }

        self.elements.push(ElementOrMarker::Element(element));
    }

    pub fn contains(&self, element: &Rc<RefCell<Element>>) -> bool {
        self.elements.iter().any(|el| match el {
            ElementOrMarker::Element(e) => Rc::ptr_eq(e, element),
            ElementOrMarker::Marker => false,
        })
    }

    pub fn reconstruct(&mut self, parser: &mut Parser) {
        parser._reconstruct_active_formatting_elements();
    }
}
