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

// impl BackgroundLayer {
//     fn _parse_step(cvs: &mut InputStream<ComponentValue>) -> Option<BackgroundLayer> {
//         println!("CVS: {:?}", cvs);
//         if let Some(image) = Image::from_cv(cvs) {
//             return Some(BackgroundLayer::Image(image));
//         }
//
//         if let Some(position) = Position::from_cv(cvs) {
//             return Some(BackgroundLayer::Position(position));
//         }
//
//         if let Some(repeat_style) = RepeatStyle::from_cv(cvs) {
//             return Some(BackgroundLayer::RepeatStyle(repeat_style));
//         }
//
//         None
//     }
// }

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

// impl CSSParseable for BackgroundLayer {
//     /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/background#formal_syntax
//     fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
//         let mut bg = BackgroundLayer::default();
//
//         let vec = cvs.finish();
//
//         let bg_layers = Background::preprocess_tokens(&vec);
//
//         for (i, layer) in bg_layers.iter().enumerate() {
//             if i == bg_layers.len() - 1 {
//                 println!("Parsing final bg layer: {:?}", layer);
//                 let mut layer_cvs = InputStream::new(layer);
//
//                 bg.update_from_layer(Background::parse_final_bg_layer(&mut layer_cvs));
//             } else {
//                 println!("Parsing bg layer: {:?}", layer);
//                 let mut layer_cvs = InputStream::new(layer);
//
//                 if let Some(parsed_layer) = Background::parse_bg_layer(&mut layer_cvs) {
//                     bg.update_from_layer(parsed_layer);
//                 }
//             }
//         }
//
//         println!("Parsed background: {:?}", bg);
//         Some(bg)
//     }
// }

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

    // fn update_from_bg(&mut self, bg: Background) {
    //     match bg {
    //         BackgroundLayer::Image(image) => {
    //             self.image = image;
    //         }
    //         BackgroundLayer::Color(color) => {
    //             self.color = color;
    //         }
    //         BackgroundLayer::Position(position) => {
    //             self.position = position;
    //         }
    //         BackgroundLayer::RepeatStyle(_repeat_style) => {
    //             self.repeat_style = _repeat_style;
    //         }
    //     }
    // }

    pub fn color(&self) -> Color {
        self.layers
            .last()
            .map_or(Color::transparent(), |layer| layer.color.clone())
    }
}

impl BackgroundLayer {
    fn parse_bg_layer(
        cvs: &mut InputStream<ComponentValue>,
        is_final: bool,
    ) -> Option<BackgroundLayer> {
        let mut layer = BackgroundLayer::default();

        while !cvs.is_eof {
            println!("Next: {:?}", cvs.peek());

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
    pub direction_x: PositionDirection,
    pub direction_y: PositionDirection,

    pub offset_x: LengthPercentage,
    pub offset_y: LengthPercentage,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            direction_x: PositionDirection::Center,
            direction_y: PositionDirection::Center,
            offset_x: LengthPercentage::Length(Dimension {
                value: 0.0,
                number_type: NumberType::Integer,
                unit: "px".to_string(),
            }),
            offset_y: LengthPercentage::Length(Dimension {
                value: 0.0,
                number_type: NumberType::Integer,
                unit: "px".to_string(),
            }),
        }
    }
}

impl CSSParseable for Position {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "left" => Some(Position {
                        direction_x: PositionDirection::Left,
                        ..Default::default()
                    }),
                    "center" => Some(Position {
                        direction_x: PositionDirection::Center,
                        direction_y: PositionDirection::Center,
                        ..Default::default()
                    }),
                    "right" => Some(Position {
                        direction_x: PositionDirection::Right,
                        ..Default::default()
                    }),
                    "top" => Some(Position {
                        direction_y: PositionDirection::Top,
                        ..Default::default()
                    }),
                    "bottom" => Some(Position {
                        direction_y: PositionDirection::Bottom,
                        ..Default::default()
                    }),
                    _ => {
                        cvs.reconsume();
                        None
                    }
                },
                ComponentValue::Token(CSSToken::Percentage(perc)) => Some(Position {
                    direction_x: PositionDirection::Center,
                    direction_y: PositionDirection::Center,
                    offset_x: LengthPercentage::Percentage(perc),
                    offset_y: LengthPercentage::Percentage(perc),
                }),
                ComponentValue::Token(CSSToken::Dimension(dim)) => Some(Position {
                    direction_x: PositionDirection::Center,
                    direction_y: PositionDirection::Center,
                    offset_x: LengthPercentage::Length(dim.clone()),
                    offset_y: LengthPercentage::Length(dim.clone()),
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
