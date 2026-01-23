use crate::{
    css::{
        colors::Color,
        parser::{ComponentValue, Function},
        tokenize::{CSSToken, Dimension, NumberType, Percentage},
    },
    html5::dom::Element,
    infra::InputStream,
};

pub trait CSSParseable {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Default, Debug, Clone)]
pub enum WidthValue {
    Length(Dimension),
    Percentage(Percentage),

    #[default]
    Auto,

    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl CSSParseable for WidthValue {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "auto" => Some(WidthValue::Auto),
                    "max-content" => Some(WidthValue::MaxContent),
                    "min-content" => Some(WidthValue::MinContent),
                    "fit-content" => Some(WidthValue::FitContent),
                    "stretch" => Some(WidthValue::Stretch),
                    _ => {
                        cvs.reconsume();
                        None
                    }
                },
                ComponentValue::Token(CSSToken::Dimension(dim)) => {
                    Some(WidthValue::Length(dim.clone()))
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    Some(WidthValue::Percentage(perc.clone()))
                }
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

impl WidthValue {
    pub fn resolve(&self, parent_width: f64) -> f64 {
        match self {
            WidthValue::Length(dim) => match dim.unit.as_str() {
                "px" => dim.value as f64,
                _ => todo!("Handle other length units"),
            },
            WidthValue::Percentage(perc) => (*perc as f64 / 100.0) * parent_width,
            WidthValue::Auto => parent_width,
            _ => todo!("Handle other WidthValue variants"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Image {
    FromUrl(String),
    None,
}

impl CSSParseable for Image {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        match cvs.peek() {
            Some(tok) => match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) if ident == "none" => {
                    cvs.consume();
                    Some(Image::None)
                }
                _ => Image::parse_definite_image(cvs),
            },
            None => None,
        }
    }
}

impl Image {
    fn parse_definite_image(cvs: &mut InputStream<ComponentValue>) -> Option<Image> {
        if let Some(url) = parse_url_function(cvs) {
            return Some(Image::FromUrl(url));
        }

        None
    }

    pub fn parse_multiple_images(cvs: &mut InputStream<ComponentValue>) -> Vec<Image> {
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

        let mut images = Vec::new();

        while let Some(image) = Image::from_cv(&mut cvs) {
            images.push(image);
        }

        images
    }
}

#[derive(Default, Debug, Clone)]
pub struct Background {
    pub layers: Vec<BackgroundLayer>,
}

#[derive(Debug, Clone)]
pub struct BackgroundLayer {
    pub image: Image,
    pub color: Color,
    pub position: Position,
    pub repeat_style: RepeatStyle,
    pub origin: Origin,
}

impl Default for BackgroundLayer {
    fn default() -> Self {
        BackgroundLayer {
            image: Image::None,
            color: Color::transparent(),
            position: Position::default(),
            repeat_style: RepeatStyle::Repeat,
            origin: Origin::PaddingBox,
        }
    }
}

impl CSSParseable for Background {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        let mut bg = Background { layers: Vec::new() };

        let vec = cvs.finish();

        let bg_layers = Background::preprocess_tokens(&vec);

        for (i, layer) in bg_layers.iter().enumerate() {
            if i == bg_layers.len() - 1 {
                let mut layer_cvs = InputStream::new(layer);

                if let Some(parsed_layer) = BackgroundLayer::parse_bg_layer(&mut layer_cvs, true) {
                    bg.layers.push(parsed_layer);
                }
            } else {
                let mut layer_cvs = InputStream::new(layer);

                if let Some(parsed_layer) = BackgroundLayer::parse_bg_layer(&mut layer_cvs, false) {
                    bg.layers.push(parsed_layer);
                }
            }
        }

        println!("Parsed background: {:#?}", bg);
        Some(bg)
    }
}

impl Background {
    pub fn preprocess_tokens(cvs: &[ComponentValue]) -> Vec<Vec<ComponentValue>> {
        cvs.iter()
            .filter(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Whitespace => false,
                    _ => true,
                },
                _ => true,
            })
            .cloned()
            .collect::<Vec<_>>()
            .split(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Comma => true,
                    _ => false,
                },
                _ => false,
            })
            .map(|slice| slice.to_vec())
            .collect()
    }

    pub fn color(&self) -> Color {
        self.layers
            .last()
            .map_or(Color::transparent(), |layer| layer.color.clone())
    }

    fn ensure_layer(&mut self) -> &mut BackgroundLayer {
        if self.layers.is_empty() {
            self.layers.push(BackgroundLayer::default());
        }
        self.layers.last_mut().unwrap()
    }

    fn update_color(&mut self, color: Color) {
        let layer = self.ensure_layer();
        layer.color = color;
    }

    pub fn set_color(&mut self, color: Color) {
        let layer = self.ensure_layer();
        layer.color = color;
    }

    pub fn set_images(&mut self, images: Vec<Image>) {
        for (i, image) in images.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].image = image;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.image = image;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }

    pub fn set_positions(&mut self, positions: Vec<Position>) {
        for (i, position) in positions.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].position = position;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.position = position;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }

    pub fn set_repeat_styles(&mut self, repeat_styles: Vec<RepeatStyle>) {
        for (i, repeat_style) in repeat_styles.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].repeat_style = repeat_style;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.repeat_style = repeat_style;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }

    pub fn set_origins(&mut self, origins: Vec<Origin>) {
        for (i, origin) in origins.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].origin = origin;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.origin = origin;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }
}

impl BackgroundLayer {
    fn parse_bg_layer(
        cvs: &mut InputStream<ComponentValue>,
        is_final: bool,
    ) -> Option<BackgroundLayer> {
        let mut layer = BackgroundLayer::default();

        while !cvs.is_eof {
            if let Some(image) = Image::from_cv(cvs) {
                layer.image = image;
                continue;
            }

            if let Some(position) = Position::from_cv(cvs) {
                layer.position = position;
                continue;
            }

            if let Some(repeat_style) = RepeatStyle::from_cv(cvs) {
                layer.repeat_style = repeat_style;
                continue;
            }

            if let Some(origin) = Origin::from_cv(cvs) {
                layer.origin = origin;
                continue;
            }

            if is_final {
                if let Some(color) = Color::from_cv(cvs) {
                    layer.color = color;
                    continue;
                }
            }
        }

        println!("Parsed background layer: {:?}", layer);
        Some(layer)
    }
}

fn parse_url_function(cvs: &mut InputStream<ComponentValue>) -> Option<String> {
    if let Some(ComponentValue::Function(Function(func_name, func_args))) = &cvs.peek() {
        if func_name == "url" {
            if let ComponentValue::Token(CSSToken::String(url)) = &func_args[0] {
                cvs.consume();
                return Some(url.clone());
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
pub enum LengthPercentage {
    Length(Dimension),
    Percentage(Percentage),
}

#[derive(Debug, Clone)]
pub enum PositionDirection {
    Left,
    Center,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: (PositionDirection, LengthPercentage),
    pub y: (PositionDirection, LengthPercentage),
}

impl Default for Position {
    fn default() -> Self {
        Position {
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
        }
    }
}

impl CSSParseable for Position {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "left" => Some(Position {
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
                    "center" => Some(Position {
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
                    }),
                    "right" => Some(Position {
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
                    "top" => Some(Position {
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
                    "bottom" => Some(Position {
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
                ComponentValue::Token(CSSToken::Percentage(perc)) => Some(Position {
                    x: (
                        PositionDirection::Center,
                        LengthPercentage::Percentage(perc.clone()),
                    ),
                    y: (
                        PositionDirection::Center,
                        LengthPercentage::Percentage(perc.clone()),
                    ),
                }),
                ComponentValue::Token(CSSToken::Dimension(dim)) => Some(Position {
                    x: (
                        PositionDirection::Center,
                        LengthPercentage::Length(dim.clone()),
                    ),
                    y: (
                        PositionDirection::Center,
                        LengthPercentage::Length(dim.clone()),
                    ),
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

impl Position {
    pub fn parse_multiple_positions(cvs: &mut InputStream<ComponentValue>) -> Vec<Position> {
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

        while let Some(position) = Position::from_cv(&mut cvs) {
            positions.push(position);
        }

        positions
    }
}

#[derive(Debug, Clone)]
pub enum RepeatStyle {
    RepeatX,
    RepeatY,
    RepeatBlock,
    RepeatInline,

    /* Repetition */
    Repeat,
    Space,
    Round,
    NoRepeat,
}

impl CSSParseable for RepeatStyle {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "repeat-x" => Some(RepeatStyle::RepeatX),
                    "repeat-y" => Some(RepeatStyle::RepeatY),
                    "repeat-block" => Some(RepeatStyle::RepeatBlock),
                    "repeat-inline" => Some(RepeatStyle::RepeatInline),
                    "repeat" => Some(RepeatStyle::Repeat),
                    "space" => Some(RepeatStyle::Space),
                    "round" => Some(RepeatStyle::Round),
                    "no-repeat" => Some(RepeatStyle::NoRepeat),
                    _ => {
                        cvs.reconsume();
                        None
                    }
                },
                _ => {
                    cvs.reconsume();
                    None
                }
            }
        } else {
            None
            // todo!("Handle more complex repeat-style parsing")
        }
    }
}

impl RepeatStyle {
    pub fn parse_multiple_repeat_styles(cvs: &mut InputStream<ComponentValue>) -> Vec<RepeatStyle> {
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

        let mut repeat_styles = Vec::new();

        while let Some(repeat_style) = RepeatStyle::from_cv(&mut cvs) {
            repeat_styles.push(repeat_style);
        }

        repeat_styles
    }
}

#[derive(Debug, Clone)]
pub enum Origin {
    PaddingBox,
    BorderBox,
    ContentBox,
}

impl CSSParseable for Origin {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "padding-box" => Some(Origin::PaddingBox),
                    "border-box" => Some(Origin::BorderBox),
                    "content-box" => Some(Origin::ContentBox),
                    _ => {
                        cvs.reconsume();
                        None
                    }
                },
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

impl Origin {
    pub fn parse_multiple_origins(cvs: &mut InputStream<ComponentValue>) -> Vec<Origin> {
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

        let mut origins = Vec::new();

        while let Some(origin) = Origin::from_cv(&mut cvs) {
            origins.push(origin);
        }

        origins
    }
}

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

    pub fn resolved_font_size(&self) -> Option<f64> {
        match self {
            Font::Constructed(cf) => cf.resolved_font_size(),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolve_font_size(&mut self, parents: &Vec<&Element>) -> Option<f64> {
        match self {
            Font::Constructed(cf) => Some(cf.resolve_font_size(parents)),
            Font::SystemFont(_) => None,
        }
    }

    pub fn resolved_line_height(&self) -> Option<f64> {
        match self {
            Font::Constructed(cf) => match &cf.line_height {
                LineHeight::Normal => Some(cf.resolved_font_size().map_or(16.0, |fs| fs * 1.2)),
                LineHeight::Number(n) => cf.resolved_font_size().map(|fs| fs * n),
                LineHeight::LengthPercentage(lp) => match lp {
                    LengthPercentage::Length(dim) => match dim.unit.as_str() {
                        "px" => Some(dim.value as f64),
                        _ => None,
                    },
                    LengthPercentage::Percentage(perc) => cf
                        .resolved_font_size()
                        .map(|fs| (*perc as f64 / 100.0) * fs),
                },
            },
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

        println!("Parsed constructed font so far: {:#?}", font);
        Some(font)
    }
}

impl ConstructedFont {
    pub fn resolve_font_size(&mut self, parents: &Vec<&Element>) -> f64 {
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
            println!("Parsing FontStyle from token: {:?}", tok);
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
pub enum FontWeight {
    #[default]
    Normal,
    Bold,

    Bolder,
    Lighter,

    Weight(u32),
}

impl CSSParseable for FontWeight {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "normal" => return Some(FontWeight::Normal),
                    "bold" => return Some(FontWeight::Bold),
                    "bolder" => return Some(FontWeight::Bolder),
                    "lighter" => return Some(FontWeight::Lighter),
                    _ => {}
                },
                ComponentValue::Token(CSSToken::Number { value, .. }) => {
                    return Some(FontWeight::Weight(value as u32));
                }
                _ => {}
            }
        }

        cvs.reconsume();
        None
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
pub enum FontSize {
    LengthPercentage(LengthPercentage),
    AbsoluteSize(AbsoluteSize),
    RelativeSize(RelativeSize),
}

impl FontSize {
    pub fn resolve(&self, parents: &Vec<&Element>) -> f64 {
        println!(
            "Parents: {:?}",
            parents
                .iter()
                .map(|e| e.local_name.clone())
                .collect::<Vec<_>>()
        );

        match self {
            FontSize::LengthPercentage(lp) => match lp {
                LengthPercentage::Length(dim) => match dim.unit.as_str() {
                    "px" => dim.value as f64,
                    "em" => {
                        let parent_font_size = parents
                            .last()
                            .and_then(|parent| parent.style().font.resolved_font_size())
                            .unwrap_or(16.0);

                        dim.value as f64 * parent_font_size
                    }
                    "rem" => {
                        let root_font_size = parents
                            .first()
                            .and_then(|root| root.style().font.resolved_font_size())
                            .unwrap_or(16.0);

                        dim.value as f64 * root_font_size
                    }
                    _ => todo!("Handle other length units"),
                },
                LengthPercentage::Percentage(perc) => {
                    // For now, assume parent font size is 16px
                    let parent_font_size = 16.0;
                    (*perc as f64 / 100.0) * parent_font_size
                }
            },
            _ => todo!("Handle other FontSize variants"),
        }

        // match self {
        //     FontSize::LengthPercentage(lp) => match lp {
        //         LengthPercentage::Length(dim) => match dim.unit.as_str() {
        //             "px" => dim.value as f64,
        //             "em" => dim.value as f64 * parent_font_size,
        //             _ => todo!("Handle other length units"),
        //         },
        //         LengthPercentage::Percentage(perc) => (*perc as f64 / 100.0) * parent_font_size,
        //     },
        //     FontSize::AbsoluteSize(abs_size) => match abs_size {
        //         AbsoluteSize::XXSmall => parent_font_size * 0.578,
        //         AbsoluteSize::XSmall => parent_font_size * 0.694,
        //         AbsoluteSize::Small => parent_font_size * 0.833,
        //         AbsoluteSize::Medium => parent_font_size,
        //         AbsoluteSize::Large => parent_font_size * 1.2,
        //         AbsoluteSize::XLarge => parent_font_size * 1.44,
        //         AbsoluteSize::XXLarge => parent_font_size * 1.728,
        //     },
        //     FontSize::RelativeSize(rel_size) => match rel_size {
        //         RelativeSize::Larger => parent_font_size * 1.2,
        //         RelativeSize::Smaller => parent_font_size * 0.833,
        //     },
        // }
    }
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::AbsoluteSize(AbsoluteSize::Medium)
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
                    return Some(FontSize::LengthPercentage(LengthPercentage::Length(
                        dim.clone(),
                    )));
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    return Some(FontSize::LengthPercentage(LengthPercentage::Percentage(
                        perc.clone(),
                    )));
                }
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "xx-small" => return Some(FontSize::AbsoluteSize(AbsoluteSize::XXSmall)),
                    "x-small" => return Some(FontSize::AbsoluteSize(AbsoluteSize::XSmall)),
                    "small" => return Some(FontSize::AbsoluteSize(AbsoluteSize::Small)),
                    "medium" => return Some(FontSize::AbsoluteSize(AbsoluteSize::Medium)),
                    "large" => return Some(FontSize::AbsoluteSize(AbsoluteSize::Large)),
                    "x-large" => return Some(FontSize::AbsoluteSize(AbsoluteSize::XLarge)),
                    "xx-large" => return Some(FontSize::AbsoluteSize(AbsoluteSize::XXLarge)),
                    "larger" => return Some(FontSize::RelativeSize(RelativeSize::Larger)),
                    "smaller" => return Some(FontSize::RelativeSize(RelativeSize::Smaller)),
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
pub enum LineHeight {
    #[default]
    Normal,
    Number(f64),
    LengthPercentage(LengthPercentage),
}

impl CSSParseable for LineHeight {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) if ident == "normal" => {
                    return Some(LineHeight::Normal);
                }
                ComponentValue::Token(CSSToken::Number { value, .. }) => {
                    return Some(LineHeight::Number(value as f64));
                }
                ComponentValue::Token(CSSToken::Dimension(dim)) => {
                    return Some(LineHeight::LengthPercentage(LengthPercentage::Length(
                        dim.clone(),
                    )));
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    return Some(LineHeight::LengthPercentage(LengthPercentage::Percentage(
                        perc.clone(),
                    )));
                }
                _ => {}
            }
        }

        cvs.reconsume();
        None
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
