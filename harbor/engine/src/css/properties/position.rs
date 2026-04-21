use crate::css::{
    properties::{LengthPercentage, prop_imports},
    tokenize::{Dimension, NumberType},
};

prop_imports!();

#[derive(Debug, Clone)]
pub enum PositionDirection {
    Left,
    Center,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct PositionValue {
    pub x: (PositionDirection, LengthPercentage),
    pub y: (PositionDirection, LengthPercentage),

    _resolved_x: Option<f64>,
    _resolved_y: Option<f64>,
}

impl Default for PositionValue {
    fn default() -> Self {
        Self {
            x: (
                PositionDirection::Center,
                LengthPercentage::Length(Dimension {
                    value: 0.0,
                    number_type: NumberType::Integer,
                    unit: "px".to_string(),
                }),
            ),
            y: (
                PositionDirection::Center,
                LengthPercentage::Length(Dimension {
                    value: 0.0,
                    number_type: NumberType::Integer,
                    unit: "px".to_string(),
                }),
            ),
            _resolved_x: None,
            _resolved_y: None,
        }
    }
}

impl CSSParseable for PositionValue {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "left" => Some(Self {
                        x: (
                            PositionDirection::Left,
                            LengthPercentage::Length(Dimension {
                                value: 0.0,
                                number_type: NumberType::Integer,
                                unit: "px".to_string(),
                            }),
                        ),
                        ..Default::default()
                    }),
                    "center" => Some(Self {
                        x: (
                            PositionDirection::Center,
                            LengthPercentage::Length(Dimension {
                                value: 0.0,
                                number_type: NumberType::Integer,
                                unit: "px".to_string(),
                            }),
                        ),
                        y: (
                            PositionDirection::Center,
                            LengthPercentage::Length(Dimension {
                                value: 0.0,
                                number_type: NumberType::Integer,
                                unit: "px".to_string(),
                            }),
                        ),

                        ..Default::default()
                    }),
                    "right" => Some(Self {
                        x: (
                            PositionDirection::Right,
                            LengthPercentage::Length(Dimension {
                                value: 0.0,
                                number_type: NumberType::Integer,
                                unit: "px".to_string(),
                            }),
                        ),
                        ..Default::default()
                    }),
                    "top" => Some(Self {
                        y: (
                            PositionDirection::Top,
                            LengthPercentage::Length(Dimension {
                                value: 0.0,
                                number_type: NumberType::Integer,
                                unit: "px".to_string(),
                            }),
                        ),
                        ..Default::default()
                    }),
                    "bottom" => Some(Self {
                        y: (
                            PositionDirection::Bottom,
                            LengthPercentage::Length(Dimension {
                                value: 0.0,
                                number_type: NumberType::Integer,
                                unit: "px".to_string(),
                            }),
                        ),
                        ..Default::default()
                    }),
                    _ => {
                        cvs.reconsume();
                        None
                    }
                },
                ComponentValue::Token(CSSToken::Percentage(perc)) => Some(Self {
                    x: (
                        PositionDirection::Center,
                        LengthPercentage::Percentage(perc.clone()),
                    ),
                    y: (
                        PositionDirection::Center,
                        LengthPercentage::Percentage(perc.clone()),
                    ),
                    ..Default::default()
                }),
                ComponentValue::Token(CSSToken::Dimension(dim)) => Some(Self {
                    x: (
                        PositionDirection::Center,
                        LengthPercentage::Length(dim.clone()),
                    ),
                    y: (
                        PositionDirection::Center,
                        LengthPercentage::Length(dim.clone()),
                    ),
                    ..Default::default()
                }),
                _ => {
                    cvs.reconsume();
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Resolvable<(f64, f64)> for PositionValue {
    fn resolved(&self) -> (f64, f64) {
        (
            self._resolved_x.unwrap_or(0.0),
            self._resolved_y.unwrap_or(0.0),
        )
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> (f64, f64) {
        let res_x = self.x.1.resolve(parents);
        let res_y = self.y.1.resolve(parents);

        self._resolved_x = Some(res_x);
        self._resolved_y = Some(res_y);

        (res_x, res_y)
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> (f64, f64) {
        let res_x = self.x.1.resolve_with_curr(parents, current, viewport_size);
        let res_y = self.y.1.resolve_with_curr(parents, current, viewport_size);

        self._resolved_x = Some(res_x);
        self._resolved_y = Some(res_y);

        (res_x, res_y)
    }
}

impl PositionValue {
    pub fn parse_multiple_positions(cvs: &mut InputStream<ComponentValue>) -> Vec<Self> {
        let mut cvs = InputStream::new(
            &cvs.finish()
                .iter()
                .filter(|cv| match cv {
                    ComponentValue::Token(token) => match token {
                        CSSToken::Whitespace | CSSToken::Comma => false,
                        _ => true,
                    },
                    _ => true,
                })
                .cloned()
                .collect::<Vec<ComponentValue>>()[..],
        );

        let mut positions = Vec::new();

        while let Some(position) = Self::from_cv(&mut cvs) {
            positions.push(position);
        }

        positions
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
    // TODO: running()
}

impl CSSParseable for Position {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "static" => return Some(Position::Static),
                    "relative" => return Some(Position::Relative),
                    "absolute" => return Some(Position::Absolute),
                    "fixed" => return Some(Position::Fixed),
                    "sticky" => return Some(Position::Sticky),
                    _ => {}
                },
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

#[derive(Debug, Clone, Default)]
pub enum PositioningValueKind {
    #[default]
    Auto,
    LengthPercentage(LengthPercentage),
    // TODO: anchor(), anchor-size()
}

#[derive(Debug, Clone, Default)]
pub struct Top {
    pub kind: PositioningValueKind,

    _resolved: Option<f64>,
}

impl CSSParseable for Top {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) if ident == "auto" => {
                    return Some(Top {
                        kind: PositioningValueKind::Auto,
                        _resolved: None,
                    });
                }
                ComponentValue::Token(CSSToken::Dimension(dim)) => {
                    return Some(Top {
                        kind: PositioningValueKind::LengthPercentage(LengthPercentage::Length(
                            dim.clone(),
                        )),
                        _resolved: None,
                    });
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    return Some(Top {
                        kind: PositioningValueKind::LengthPercentage(LengthPercentage::Percentage(
                            perc.clone(),
                        )),
                        _resolved: None,
                    });
                }
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

impl Resolvable<f64> for Top {
    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let res = match &self.kind {
            PositioningValueKind::Auto => None,
            PositioningValueKind::LengthPercentage(lp) => Some(lp.resolve(parents)),
        }
        .unwrap_or(0.0);

        self._resolved = Some(res);
        res
    }

    fn resolved(&self) -> f64 {
        self._resolved.unwrap_or(0.0)
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        style: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let res = match &self.kind {
            PositioningValueKind::Auto => None,
            PositioningValueKind::LengthPercentage(lp) => {
                Some(lp.resolve_with_curr(parents, style, viewport_size))
            }
        }
        .unwrap_or(0.0);

        self._resolved = Some(res);
        res
    }
}

pub type Bottom = Top;
pub type Left = Top;
pub type Right = Top;
