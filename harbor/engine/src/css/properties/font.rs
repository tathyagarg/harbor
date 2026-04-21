use crate::css::{
    properties::{LengthPercentage, prop_imports},
    tokenize::Dimension,
};

prop_imports!();

#[derive(Debug, Clone)]
pub enum Font {
    Constructed(ConstructedFont),
    SystemFont(String),
}

impl Font {
    pub fn size(&self) -> FontSize {
        match self {
            Font::Constructed(cf) => cf.size.clone(),
            Font::SystemFont(_) => FontSize::default(),
        }
    }

    pub fn family(&self) -> FontFamily {
        match self {
            Font::Constructed(cf) => cf.family.clone(),
            Font::SystemFont(system) => FontFamily {
                entries: vec![FontFamilyEntry::GenericFamily(system.clone())],
            },
        }
    }

    pub fn weight(&self) -> FontWeight {
        match self {
            Font::Constructed(cf) => cf.weight.clone(),
            Font::SystemFont(_) => FontWeight::default(),
        }
    }

    pub fn style(&self) -> FontStyle {
        match self {
            Font::Constructed(cf) => cf.style.clone(),
            Font::SystemFont(_) => FontStyle::default(),
        }
    }

    pub fn line_height(&self) -> LineHeight {
        match self {
            Font::Constructed(cf) => cf.line_height.clone(),
            Font::SystemFont(_) => LineHeight::default(),
        }
    }

    pub fn set_size(&mut self, size: FontSize) {
        match self {
            Font::Constructed(cf) => cf.size = size,
            Font::SystemFont(_) => {}
        }
    }

    pub fn set_family(&mut self, family: FontFamily) {
        match self {
            Font::Constructed(cf) => cf.family = family,
            Font::SystemFont(_) => {}
        }
    }

    pub fn set_line_height(&mut self, line_height: LineHeight) {
        match self {
            Font::Constructed(cf) => cf.line_height = line_height,
            Font::SystemFont(_) => {}
        }
    }

    pub fn set_weight(&mut self, weight: FontWeight) {
        match self {
            Font::Constructed(cf) => cf.weight = weight,
            Font::SystemFont(_) => {}
        }
    }

    pub fn set_style(&mut self, style: FontStyle) {
        match self {
            Font::Constructed(cf) => cf.style = style,
            Font::SystemFont(_) => {}
        }
    }

    pub fn resolved_font_size(&self) -> Option<f64> {
        match self {
            Font::Constructed(cf) => cf.resolved_font_size(),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolve_font_size(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> Option<f64> {
        match self {
            Font::Constructed(cf) => Some(cf.resolve_font_size(parents)),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolve_line_height_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> Option<f64> {
        match self {
            Font::Constructed(cf) => Some(cf.line_height.resolve_with_curr(
                parents,
                current,
                viewport_size,
            )),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolve_line_height(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> Option<f64> {
        match self {
            Font::Constructed(cf) => Some(cf.line_height.resolve(parents)),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolved_line_height(&self) -> Option<f64> {
        match self {
            Font::Constructed(cf) => Some(cf.line_height.resolved()),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolve_font_weight(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> Option<u32> {
        match self {
            Font::Constructed(cf) => Some(cf.resolve_font_weight(parents)),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolved_font_weight(&self) -> Option<u32> {
        match self {
            Font::Constructed(cf) => cf.resolved_font_weight(),
            Font::SystemFont(_) => None,
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Font::Constructed(ConstructedFont::default())
    }
}

#[derive(Default, Debug, Clone)]
pub struct ConstructedFont {
    pub style: FontStyle,
    pub variant: FontVariant,
    pub weight: FontWeight,
    pub width: FontWidth,
    pub size: FontSize,
    pub line_height: LineHeight,
    pub family: FontFamily,

    _resolved_font_size: Option<f64>,
    _resolve_font_weight: Option<u32>,
}

impl CSSParseable for Font {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        let cvs_vec = cvs.finish();

        let mut cvs = InputStream::new(
            &cvs_vec
                .iter()
                .filter(|cv| match cv {
                    ComponentValue::Token(token) => match token {
                        CSSToken::Whitespace => false,
                        _ => true,
                    },
                    _ => true,
                })
                .cloned()
                .collect::<Vec<ComponentValue>>()[..],
        );

        if let Some(tok) = cvs.peek() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident))
                    if matches!(
                        ident.as_str(),
                        "caption"
                            | "icon"
                            | "menu"
                            | "message-box"
                            | "small-caption"
                            | "status-bar"
                    ) =>
                {
                    cvs.consume();
                    return Some(Font::SystemFont(ident));
                }
                _ => ConstructedFont::from_cv(&mut cvs).map(|cf| Font::Constructed(cf)),
            }
        } else {
            None
        }
    }
}

impl Resolvable<(f64, u32)> for Font {
    fn resolved(&self) -> (f64, u32) {
        (
            self.resolved_font_size().unwrap_or(DEFAULT_FONT_SIZE),
            self.resolved_font_weight().unwrap_or(400),
        )
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> (f64, u32) {
        (
            self.resolve_font_size(parents).unwrap_or(DEFAULT_FONT_SIZE),
            self.resolve_font_weight(parents).unwrap_or(400),
        )
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> (f64, u32) {
        (
            match self {
                Font::Constructed(cf) => cf.size.resolve_with_curr(parents, current, viewport_size),
                Font::SystemFont(_) => DEFAULT_FONT_SIZE,
            },
            match self {
                Font::Constructed(cf) => {
                    cf.weight.resolve_with_curr(parents, current, viewport_size)
                }
                Font::SystemFont(_) => 400,
            },
        )
    }
}

impl CSSParseable for ConstructedFont {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        let mut font = ConstructedFont::default();

        let mut progress = true;

        while progress {
            progress = false;

            if let Some(style) = FontStyle::from_cv(cvs) {
                font.style = style;
                progress = true;
                continue;
            }

            if let Some(variant) = FontVariant::from_cv(cvs) {
                font.variant = variant;
                progress = true;
                continue;
            }

            if let Some(weight) = FontWeight::from_cv(cvs) {
                font.weight = weight;
                progress = true;
                continue;
            }

            if let Some(width) = FontWidth::from_cv(cvs) {
                font.width = width;
                progress = true;
                continue;
            }
        }

        if let Some(size) = FontSize::from_cv(cvs) {
            font.size = size;

            if let Some(tok) = cvs.consume() {
                if let ComponentValue::Token(CSSToken::Delim('\u{002F}')) = tok {
                    if let Some(line_height) = LineHeight::from_cv(cvs) {
                        font.line_height = line_height;
                    }
                } else {
                    cvs.reconsume();
                }
            }
        } else {
            return None;
        }

        if let Some(family) = FontFamily::from_cv(cvs) {
            font.family = family;
        } else {
            return None;
        }

        Some(font)
    }
}

impl Resolvable<(f64, u32)> for ConstructedFont {
    fn resolved(&self) -> (f64, u32) {
        (
            self.resolved_font_size().unwrap_or(DEFAULT_FONT_SIZE),
            self.resolved_font_weight().unwrap_or(400),
        )
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> (f64, u32) {
        (
            self.resolve_font_size(parents),
            self.resolve_font_weight(parents),
        )
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> (f64, u32) {
        (
            self.size.resolve_with_curr(parents, current, viewport_size),
            self.weight
                .resolve_with_curr(parents, current, viewport_size),
        )
    }
}

impl ConstructedFont {
    pub fn resolve_font_size(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        // if let Some(resolved_size) = self._resolved_font_size {
        //     return resolved_size;
        // }

        let resolved_size = self.size.resolve(parents);
        self._resolved_font_size = Some(resolved_size);
        resolved_size
    }

    pub fn resolved_font_size(&self) -> Option<f64> {
        self._resolved_font_size
    }

    pub fn resolve_font_weight(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> u32 {
        if let Some(resolved_weight) = self._resolve_font_weight {
            return resolved_weight;
        }

        let resolved_weight = self.weight.resolve(parents);
        self._resolve_font_weight = Some(resolved_weight);
        resolved_weight
    }

    pub fn resolved_font_weight(&self) -> Option<u32> {
        self._resolve_font_weight
    }
}

#[derive(Default, Debug, Clone)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique(Option<Dimension>),
    Left,
    Right,
}

impl CSSParseable for FontStyle {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "normal" => return Some(FontStyle::Normal),
                    "italic" => return Some(FontStyle::Italic),
                    "oblique" => {
                        if let Some(ComponentValue::Token(CSSToken::Dimension(dim))) = cvs.peek() {
                            cvs.consume();
                            return Some(FontStyle::Oblique(Some(dim.clone())));
                        } else {
                            return Some(FontStyle::Oblique(None));
                        }
                    }
                    "left" => return Some(FontStyle::Left),
                    "right" => return Some(FontStyle::Right),
                    _ => {}
                },
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

#[derive(Default, Debug, Clone)]
pub enum FontVariant {
    #[default]
    Normal,

    SmallCaps,
}

impl CSSParseable for FontVariant {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "normal" => return Some(FontVariant::Normal),
                    "small-caps" => return Some(FontVariant::SmallCaps),
                    _ => {}
                },
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

#[derive(Default, Debug, Clone)]
pub enum FontWeightKind {
    #[default]
    Normal,
    Bold,

    Bolder,
    Lighter,

    Weight(u32),
}

#[derive(Debug, Clone, Default)]
pub struct FontWeight {
    pub kind: FontWeightKind,

    _resolved_weight: Option<u32>,
}

impl FontWeight {
    pub fn new(kind: FontWeightKind) -> Self {
        FontWeight {
            kind,
            _resolved_weight: None,
        }
    }
}

impl CSSParseable for FontWeight {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "normal" => return Some(FontWeight::new(FontWeightKind::Normal)),
                    "bold" => return Some(FontWeight::new(FontWeightKind::Bold)),
                    "bolder" => return Some(FontWeight::new(FontWeightKind::Bolder)),
                    "lighter" => return Some(FontWeight::new(FontWeightKind::Lighter)),
                    _ => {}
                },
                ComponentValue::Token(CSSToken::Number { value, .. }) => {
                    return Some(FontWeight::new(FontWeightKind::Weight(value as u32)));
                }
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

impl Resolvable<u32> for FontWeight {
    fn resolved(&self) -> u32 {
        self._resolved_weight.unwrap_or(400)
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> u32 {
        let res = match self.kind {
            FontWeightKind::Normal => 400,
            FontWeightKind::Bold => 700,
            FontWeightKind::Bolder => {
                let new_parents = if parents.len() > 1 {
                    &parents[..parents.len() - 1].to_vec()
                } else {
                    &vec![]
                };

                let parent_weight = parents
                    .last()
                    .and_then(|parent| {
                        parent
                            .borrow()
                            .style()
                            .font
                            .weight()
                            .resolve(new_parents)
                            .into()
                    })
                    .unwrap_or(400);

                match parent_weight {
                    100..=300 => 400,
                    400..=600 => 700,
                    700..=900 => 900,
                    _ => 400,
                }
            }
            FontWeightKind::Lighter => {
                let new_parents = if parents.len() > 1 {
                    &parents[..parents.len() - 1].to_vec()
                } else {
                    &vec![]
                };

                let parent_weight = parents
                    .last()
                    .and_then(|parent| {
                        parent
                            .borrow()
                            .style()
                            .font
                            .weight()
                            .resolve(new_parents)
                            .into()
                    })
                    .unwrap_or(400);

                match parent_weight {
                    100..=300 => 100,
                    400..=600 => 300,
                    700..=900 => 600,
                    _ => 400,
                }
            }
            FontWeightKind::Weight(w) => w,
        };

        self._resolved_weight = Some(res);
        res
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        _current: &ComputedStyle,
        _viewport_size: (f64, f64),
    ) -> u32 {
        self.resolve(parents)
    }
}

#[derive(Default, Debug, Clone)]
pub enum FontWidth {
    #[default]
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl CSSParseable for FontWidth {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "normal" => return Some(FontWidth::Normal),
                    "ultra-condensed" => return Some(FontWidth::UltraCondensed),
                    "extra-condensed" => return Some(FontWidth::ExtraCondensed),
                    "condensed" => return Some(FontWidth::Condensed),
                    "semi-condensed" => return Some(FontWidth::SemiCondensed),
                    "semi-expanded" => return Some(FontWidth::SemiExpanded),
                    "expanded" => return Some(FontWidth::Expanded),
                    "extra-expanded" => return Some(FontWidth::ExtraExpanded),
                    "ultra-expanded" => return Some(FontWidth::UltraExpanded),
                    _ => {}
                },
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

#[derive(Debug, Clone)]
pub enum FontSizeKind {
    LengthPercentage(LengthPercentage),
    AbsoluteSize(AbsoluteSize),
    RelativeSize(RelativeSize),
}

#[derive(Debug, Clone)]
pub struct FontSize {
    pub kind: FontSizeKind,

    _resolved_size: Option<f64>,
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize {
            kind: FontSizeKind::AbsoluteSize(AbsoluteSize::Medium),
            _resolved_size: None,
        }
    }
}

impl FontSize {
    pub fn new(kind: FontSizeKind) -> Self {
        FontSize {
            kind,
            _resolved_size: None,
        }
    }
}

impl CSSParseable for FontSize {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Dimension(dim)) => {
                    return Some(FontSize::new(FontSizeKind::LengthPercentage(
                        LengthPercentage::Length(dim.clone()),
                    )));
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    return Some(FontSize::new(FontSizeKind::LengthPercentage(
                        LengthPercentage::Percentage(perc.clone()),
                    )));
                }
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "xx-small" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::XXSmall,
                        )));
                    }
                    "x-small" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::XSmall,
                        )));
                    }
                    "small" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::Small,
                        )));
                    }
                    "medium" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::Medium,
                        )));
                    }
                    "large" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::Large,
                        )));
                    }
                    "x-large" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::XLarge,
                        )));
                    }
                    "xx-large" => {
                        return Some(FontSize::new(FontSizeKind::AbsoluteSize(
                            AbsoluteSize::XXLarge,
                        )));
                    }
                    "larger" => {
                        return Some(FontSize::new(FontSizeKind::RelativeSize(
                            RelativeSize::Larger,
                        )));
                    }
                    "smaller" => {
                        return Some(FontSize::new(FontSizeKind::RelativeSize(
                            RelativeSize::Smaller,
                        )));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

impl Resolvable<f64> for FontSize {
    fn resolved(&self) -> f64 {
        self._resolved_size.unwrap_or(DEFAULT_FONT_SIZE)
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let res = match &self.kind {
            FontSizeKind::LengthPercentage(lp) => match lp {
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
                    let parent_font_size = parents
                        .last()
                        .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                        .unwrap_or(DEFAULT_FONT_SIZE);
                    (*perc as f64 / 100.0) * parent_font_size
                }
            },
            FontSizeKind::RelativeSize(RelativeSize::Larger) => {
                let parent_font_size = parents
                    .last()
                    .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                    .unwrap_or(DEFAULT_FONT_SIZE);

                parent_font_size * 1.2
            }
            FontSizeKind::RelativeSize(RelativeSize::Smaller) => {
                let parent_font_size = parents
                    .last()
                    .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                    .unwrap_or(DEFAULT_FONT_SIZE);

                parent_font_size * 0.833
            }
            _ => todo!("Handle other FontSize variants"),
        };

        self._resolved_size = Some(res);
        res
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        _current: &ComputedStyle,
        _viewport_size: (f64, f64),
    ) -> f64 {
        self.resolve(parents)
    }
}

#[derive(Default, Debug, Clone)]
pub enum AbsoluteSize {
    XXSmall,
    XSmall,
    Small,

    #[default]
    Medium,

    Large,
    XLarge,
    XXLarge,
}

#[derive(Debug, Clone)]
pub enum RelativeSize {
    Larger,
    Smaller,
}

#[derive(Default, Debug, Clone)]
pub enum LineHeightKind {
    #[default]
    Normal,
    Number(f64),
    LengthPercentage(LengthPercentage),
}

#[derive(Debug, Clone, Default)]
pub struct LineHeight {
    pub kind: LineHeightKind,

    _resolved_line_height: Option<f64>,
}

impl LineHeight {
    pub fn new(kind: LineHeightKind) -> Self {
        LineHeight {
            kind,
            _resolved_line_height: None,
        }
    }
}

impl CSSParseable for LineHeight {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) if ident == "normal" => {
                    return Some(LineHeight::new(LineHeightKind::Normal));
                }
                ComponentValue::Token(CSSToken::Number { value, .. }) => {
                    return Some(LineHeight::new(LineHeightKind::Number(value as f64)));
                }
                ComponentValue::Token(CSSToken::Dimension(dim)) => {
                    return Some(LineHeight::new(LineHeightKind::LengthPercentage(
                        LengthPercentage::Length(dim.clone()),
                    )));
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    return Some(LineHeight::new(LineHeightKind::LengthPercentage(
                        LengthPercentage::Percentage(perc.clone()),
                    )));
                }
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

impl Resolvable<f64> for LineHeight {
    fn resolved(&self) -> f64 {
        self._resolved_line_height.unwrap_or(DEFAULT_FONT_SIZE)
    }

    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> f64 {
        let res = match &self.kind {
            LineHeightKind::Normal => {
                let font_size = parents
                    .last()
                    .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                    .unwrap_or(DEFAULT_FONT_SIZE);
                font_size * 1.2
            }
            LineHeightKind::Number(n) => {
                let font_size = parents
                    .last()
                    .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                    .unwrap_or(DEFAULT_FONT_SIZE);
                font_size * n
            }
            LineHeightKind::LengthPercentage(lp) => match lp {
                LengthPercentage::Length(dim) => match dim.unit.as_str() {
                    "px" => dim.value as f64,
                    _ => todo!("Handle other length units"),
                },
                LengthPercentage::Percentage(perc) => {
                    let font_size = parents
                        .last()
                        .and_then(|parent| parent.borrow().style().font.resolved_font_size())
                        .unwrap_or(DEFAULT_FONT_SIZE);
                    (*perc as f64 / 100.0) * font_size
                }
            },
        };

        self._resolved_line_height = Some(res);
        res
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> f64 {
        let res = match &self.kind {
            LineHeightKind::Normal => {
                let font_size = current
                    .font
                    .resolved_font_size()
                    .unwrap_or(DEFAULT_FONT_SIZE);
                font_size * 1.2
            }
            LineHeightKind::Number(n) => {
                let font_size = current
                    .font
                    .resolved_font_size()
                    .unwrap_or(DEFAULT_FONT_SIZE);
                font_size * n
            }
            LineHeightKind::LengthPercentage(lp) => {
                lp.resolve_with_curr(parents, current, viewport_size)
            }
        };

        self._resolved_line_height = Some(res);
        res
    }
}

#[derive(Default, Debug, Clone)]
pub struct FontFamily {
    pub entries: Vec<FontFamilyEntry>,
}

impl CSSParseable for FontFamily {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        let mut family = FontFamily {
            entries: Vec::new(),
        };

        let cvs_vec = cvs.finish();

        let mut families_cvs = cvs_vec
            .split(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Comma => true,
                    _ => false,
                },
                _ => false,
            })
            .map(|slice| slice.to_vec())
            .collect::<Vec<Vec<ComponentValue>>>();

        for fam_tokens in families_cvs.iter_mut() {
            let mut fam_cvs = InputStream::new(&fam_tokens[..]);

            if let Some(tok) = fam_cvs.consume() {
                match tok {
                    ComponentValue::Token(CSSToken::Ident(ident))
                        if matches!(
                            ident.as_str(),
                            "serif"
                                | "sans-serif"
                                | "monospace"
                                | "cursive"
                                | "fantasy"
                                | "system-ui"
                        ) =>
                    {
                        family.entries.push(FontFamilyEntry::GenericFamily(ident));
                    }
                    ComponentValue::Token(CSSToken::String(fam_name)) => {
                        family
                            .entries
                            .push(FontFamilyEntry::FamilyName(FamilyName::String(fam_name)));
                    }
                    ComponentValue::Token(CSSToken::Ident(ident)) => {
                        let mut idents = vec![ident];

                        while let Some(ComponentValue::Token(CSSToken::Ident(next_ident))) =
                            fam_cvs.peek()
                        {
                            fam_cvs.consume();
                            idents.push(next_ident);
                        }

                        family
                            .entries
                            .push(FontFamilyEntry::FamilyName(FamilyName::Idents(idents)));
                    }
                    _ => {
                        fam_cvs.reconsume();
                    }
                }
            }
        }

        Some(family)
    }
}

#[derive(Debug, Clone)]
pub enum FontFamilyEntry {
    FamilyName(FamilyName),
    GenericFamily(String),
}

impl FontFamilyEntry {
    pub fn value(&self) -> String {
        match self {
            FontFamilyEntry::FamilyName(fam_name) => match fam_name {
                FamilyName::String(s) => s.clone(),
                FamilyName::Idents(idents) => idents.join(" "),
            },
            FontFamilyEntry::GenericFamily(generic) => generic.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FamilyName {
    String(String),
    Idents(Vec<String>),
}
