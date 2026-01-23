use crate::{
    css::{
        colors::Color,
        parser::{ComponentValue, Function},
        tokenize::{CSSToken, Dimension, NumberType, Percentage},
    },
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
