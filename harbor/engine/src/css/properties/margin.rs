use crate::css::{
    r#box::Edges,
    properties::{LengthPercentage, prop_imports},
    tokenize::{Dimension, NumberType},
};

prop_imports!();

#[derive(Debug, Clone)]
pub enum MarginValueKind {
    LengthPercentage(LengthPercentage),
    Auto,
}

#[derive(Debug, Clone)]
pub struct MarginValue {
    pub kind: MarginValueKind,

    _resolved: Option<f64>,
}

impl Default for MarginValue {
    fn default() -> Self {
        MarginValue {
            kind: MarginValueKind::LengthPercentage(LengthPercentage::Length(Dimension {
                value: 0.0,
                number_type: NumberType::Integer,
                unit: "px".to_string(),
            })),
            _resolved: None,
        }
    }
}

impl MarginValue {
    pub fn new(kind: MarginValueKind) -> Self {
        MarginValue {
            kind,
            _resolved: None,
        }
    }
}

impl CSSParseable for MarginValue {
    fn from_cv(stream: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(next) = stream.consume() {
            match next {
                ComponentValue::Token(CSSToken::Number { value: 0.0, .. }) => {
                    Some(MarginValue::new(MarginValueKind::LengthPercentage(
                        LengthPercentage::Length(Dimension {
                            value: 0.0,
                            number_type: NumberType::Integer,
                            unit: "px".to_string(),
                        }),
                    )))
                }
                ComponentValue::Token(CSSToken::Dimension(dim)) => Some(MarginValue::new(
                    MarginValueKind::LengthPercentage(LengthPercentage::Length(dim)),
                )),
                ComponentValue::Token(CSSToken::Percentage(perc)) => Some(MarginValue::new(
                    MarginValueKind::LengthPercentage(LengthPercentage::Percentage(perc)),
                )),
                ComponentValue::Token(CSSToken::Ident(ident)) if ident == "auto" => {
                    Some(MarginValue::new(MarginValueKind::Auto))
                }
                _ => {
                    stream.reconsume();
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Resolvable<f64> for MarginValue {
    fn resolved(&self) -> f64 {
        self._resolved.unwrap_or(0.0)
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let res = match &self.kind {
            MarginValueKind::LengthPercentage(lp) => match lp {
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
                LengthPercentage::Percentage(_) => {
                    panic!("Idk how to resolve percentage margins yet")
                }
            },
            MarginValueKind::Auto => 0.0,
        };

        self._resolved = Some(res);
        res
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let res = match &self.kind {
            MarginValueKind::LengthPercentage(lp) => {
                lp.resolve_with_curr(parents, style, viewport_size)
            }
            MarginValueKind::Auto => 0.0,
        };

        self._resolved = Some(res);
        res
    }
}

#[derive(Debug, Clone)]
pub struct Margin {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,

    _resolved: Option<Edges>,
}

impl Default for Margin {
    fn default() -> Self {
        Margin {
            top: MarginValue::default(),
            right: MarginValue::default(),
            bottom: MarginValue::default(),
            left: MarginValue::default(),
            _resolved: None,
        }
    }
}

impl Margin {
    pub fn resolve_top(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let resolved_top = self.top.resolve(parents);

        self._resolved.get_or_insert_with(|| Edges::default()).0 = resolved_top;
        resolved_top
    }

    pub fn resolve_right(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let resolved_right = self.right.resolve(parents);

        self._resolved.get_or_insert_with(|| Edges::default()).1 = resolved_right;
        resolved_right
    }

    pub fn resolve_bottom(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let resolved_bottom = self.bottom.resolve(parents);

        self._resolved.get_or_insert_with(|| Edges::default()).2 = resolved_bottom;
        resolved_bottom
    }

    pub fn resolve_left(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let resolved_left = self.left.resolve(parents);

        self._resolved.get_or_insert_with(|| Edges::default()).3 = resolved_left;
        resolved_left
    }

    pub fn resolve_top_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let resolved_top = self.top.resolve_with_curr(parents, style, viewport_size);

        self._resolved.get_or_insert_with(|| Edges::default()).0 = resolved_top;
        resolved_top
    }

    pub fn resolve_right_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let resolved_right = self.right.resolve_with_curr(parents, style, viewport_size);

        self._resolved.get_or_insert_with(|| Edges::default()).1 = resolved_right;
        resolved_right
    }

    pub fn resolve_bottom_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let resolved_bottom = self.bottom.resolve_with_curr(parents, style, viewport_size);

        self._resolved.get_or_insert_with(|| Edges::default()).2 = resolved_bottom;
        resolved_bottom
    }

    pub fn resolve_left_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let resolved_left = self.left.resolve_with_curr(parents, style, viewport_size);

        self._resolved.get_or_insert_with(|| Edges::default()).3 = resolved_left;
        resolved_left
    }

    // pub fn egdes(&self) -> Edges {
    //     self._resolved.clone().unwrap_or(Edges::default())
    // }

    // pub fn to_edges(&mut self, parents: &Vec<Weak<RefCell<Box>>>) {
    //     self.resolve_top(parents);
    //     self.resolve_right(parents);
    //     self.resolve_bottom(parents);
    //     self.resolve_left(parents);
    // }
}

impl CSSParseable for Margin {
    fn from_cv(stream: &mut InputStream<ComponentValue>) -> Option<Self> {
        let mut values: Vec<MarginValue> = vec![];

        while !stream.is_eof {
            let next = stream.peek();

            if let Some(ComponentValue::Token(CSSToken::Whitespace)) = next {
                stream.consume();
                continue;
            }

            if let Some(margin_val) = MarginValue::from_cv(stream) {
                values.push(margin_val);
            } else {
                break;
            }
        }

        match values.len() {
            1 => Some(Margin {
                top: values[0].clone(),
                right: values[0].clone(),
                bottom: values[0].clone(),
                left: values[0].clone(),
                _resolved: None,
            }),
            2 => Some(Margin {
                top: values[0].clone(),
                right: values[1].clone(),
                bottom: values[0].clone(),
                left: values[1].clone(),
                _resolved: None,
            }),
            3 => Some(Margin {
                top: values[0].clone(),
                right: values[1].clone(),
                bottom: values[2].clone(),
                left: values[1].clone(),
                _resolved: None,
            }),
            4 => Some(Margin {
                top: values[0].clone(),
                right: values[1].clone(),
                bottom: values[2].clone(),
                left: values[3].clone(),
                _resolved: None,
            }),
            _ => None,
        }
    }
}

impl Resolvable<Edges> for Margin {
    fn resolved(&self) -> Edges {
        self._resolved.clone().unwrap_or(Edges::default())
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> Edges {
        self.resolve_top(parents);
        self.resolve_right(parents);
        self.resolve_bottom(parents);
        self.resolve_left(parents);

        self._resolved.unwrap_or_default()
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> Edges {
        self.resolve_top_with_curr(parents, style, viewport_size);
        self.resolve_right_with_curr(parents, style, viewport_size);
        self.resolve_bottom_with_curr(parents, style, viewport_size);
        self.resolve_left_with_curr(parents, style, viewport_size);

        self._resolved.unwrap_or_default()
    }
}
