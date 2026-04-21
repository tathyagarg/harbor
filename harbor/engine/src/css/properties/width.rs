use crate::css::properties::{LengthPercentage, prop_imports};

prop_imports!();

#[derive(Default, Debug, Clone)]
pub enum WidthValueKind {
    LengthPercentage(LengthPercentage),

    #[default]
    Auto,

    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

#[derive(Debug, Clone, Default)]
pub struct WidthValue {
    pub kind: WidthValueKind,

    _resolved: Option<f64>,
}

impl WidthValue {
    pub fn new(kind: WidthValueKind) -> Self {
        WidthValue {
            kind,
            _resolved: None,
        }
    }
}

impl CSSParseable for WidthValue {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "auto" => Some(WidthValue::new(WidthValueKind::Auto)),
                    "max-content" => Some(WidthValue::new(WidthValueKind::MaxContent)),
                    "min-content" => Some(WidthValue::new(WidthValueKind::MinContent)),
                    "fit-content" => Some(WidthValue::new(WidthValueKind::FitContent)),
                    "stretch" => Some(WidthValue::new(WidthValueKind::Stretch)),
                    _ => {
                        cvs.reconsume();
                        None
                    }
                },
                ComponentValue::Token(CSSToken::Dimension(dim)) => Some(WidthValue::new(
                    WidthValueKind::LengthPercentage(LengthPercentage::Length(dim.clone())),
                )),
                ComponentValue::Token(CSSToken::Percentage(perc)) => Some(WidthValue::new(
                    WidthValueKind::LengthPercentage(LengthPercentage::Percentage(perc.clone())),
                )),
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

impl Resolvable<f64> for WidthValue {
    fn resolved(&self) -> f64 {
        self._resolved.unwrap_or(0.0)
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let res = match &self.kind {
            WidthValueKind::LengthPercentage(lp) => lp.resolve(parents),
            WidthValueKind::Auto => parents
                .last()
                .and_then(|parent| parent.borrow().style().width.resolved().into())
                .unwrap_or(0.0),
            _ => todo!("Handle other WidthValue variants"),
        };

        self._resolved = Some(res);

        res
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let res = match &self.kind {
            WidthValueKind::LengthPercentage(lp) => {
                lp.resolve_with_curr(parents, current, viewport_size)
            }
            WidthValueKind::Auto => parents
                .last()
                .and_then(|parent| parent.borrow().style().width.resolved().into())
                .unwrap_or(0.0),
            _ => todo!("Handle other WidthValue variants"),
        };

        self._resolved = Some(res);
        res
    }
}

impl WidthValue {
    pub fn resolve_single_parent(&mut self, parent_width: f64) -> f64 {
        let res = match &self.kind {
            WidthValueKind::LengthPercentage(lp) => match &lp {
                LengthPercentage::Length(dim) => match dim.unit.as_str() {
                    "px" => dim.value as f64,
                    _ => todo!("Handle other length units"),
                },
                LengthPercentage::Percentage(perc) => (*perc as f64 / 100.0) * parent_width,
            },
            WidthValueKind::Auto => parent_width,
            _ => todo!("Handle other WidthValue variants"),
        };

        self._resolved = Some(res);

        res
    }
}
