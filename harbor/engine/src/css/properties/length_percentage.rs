use crate::css::{
    properties::prop_imports,
    tokenize::{Dimension, Percentage},
};

prop_imports!();

#[derive(Debug, Clone)]
pub enum LengthPercentage {
    Length(Dimension),
    Percentage(Percentage),
}

impl LengthPercentage {
    pub fn resolve(&self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        match self {
            LengthPercentage::Length(dim) => match dim.unit.as_str() {
                "px" => dim.value as f64,
                "em" => {
                    let parent_font_size = parents
                        .last()
                        .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                        .unwrap_or(DEFAULT_FONT_SIZE);

                    dim.value as f64 * parent_font_size
                }
                "rem" => {
                    let root_font_size = parents
                        .first()
                        .and_then(|root| root.borrow().style().font.resolved_font_size())
                        .unwrap_or(DEFAULT_FONT_SIZE);

                    dim.value as f64 * root_font_size
                }
                _ => todo!("Handle other length units"),
            },
            LengthPercentage::Percentage(perc) => {
                // For now, assume parent font size is 16px
                let parent_font_size = DEFAULT_FONT_SIZE;
                (*perc as f64 / 100.0) * parent_font_size
            }
        }
    }

    pub fn resolve_with_curr(
        &self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        match self {
            LengthPercentage::Length(dim) => match dim.unit.as_str() {
                "px" => dim.value as f64,
                "em" => {
                    let current_font_size = current
                        .font
                        .resolved_font_size()
                        .unwrap_or(DEFAULT_FONT_SIZE);
                    dim.value as f64 * current_font_size
                }
                "rem" => {
                    let root_font_size = parents
                        .first()
                        .and_then(|root| root.borrow().style().font.resolved_font_size())
                        .unwrap_or(DEFAULT_FONT_SIZE);

                    dim.value as f64 * root_font_size
                }
                "vw" => (dim.value as f64 / 100.0) * viewport_size.0,
                "vh" => (dim.value as f64 / 100.0) * viewport_size.1,
                _ => todo!("Handle other length units"),
            },
            LengthPercentage::Percentage(perc) => {
                // For now, assume parent font size is 16px
                let parent_font_size = DEFAULT_FONT_SIZE;
                (*perc as f64 / 100.0) * parent_font_size
            }
        }
    }
}
