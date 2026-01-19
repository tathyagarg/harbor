use crate::css::{
    colors::Color,
    parser::ComponentValue,
    tokenize::{CSSToken, Dimension, Percentage},
};

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

impl WidthValue {
    pub fn from_cv(cvs: &Vec<ComponentValue>) -> Option<Self> {
        assert!(cvs.len() == 1);

        match &cvs[0] {
            ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                "auto" => Some(WidthValue::Auto),
                "max-content" => Some(WidthValue::MaxContent),
                "min-content" => Some(WidthValue::MinContent),
                "fit-content" => Some(WidthValue::FitContent),
                "stretch" => Some(WidthValue::Stretch),
                _ => None,
            },
            ComponentValue::Token(CSSToken::Dimension(dim)) => {
                Some(WidthValue::Length(dim.clone()))
            }
            ComponentValue::Token(CSSToken::Percentage(perc)) => {
                Some(WidthValue::Percentage(perc.clone()))
            }
            _ => None,
        }
    }

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
}

#[derive(Debug, Clone)]
pub struct Background {
    pub image: Option<Image>,
    pub color: Color,
}

impl Default for Background {
    fn default() -> Self {
        Background {
            image: None,
            color: Color::transparent(),
        }
    }
}

struct BackgroundLayer {
    image: Option<Image>,
    color: Option<Color>,
}

impl Background {
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/background#formal_syntax
    pub fn from_cv(cvs: &Vec<ComponentValue>) -> Self {
        let mut bg = Self::default();

        let filtered_cvs = cvs
            .iter()
            .filter(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Whitespace => false,
                    _ => true,
                },
                _ => true,
            })
            .map(|cv| cv.clone())
            .collect::<Vec<_>>();

        let bg_layers = filtered_cvs
            .split(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Comma => true,
                    _ => false,
                },
                _ => false,
            })
            .collect::<Vec<&[ComponentValue]>>();

        for (i, layer) in bg_layers.iter().enumerate() {
            let parsed_layer = if i == bg_layers.len() - 1 {
                Self::parse_final_bg_layer(&layer)
            } else {
                Self::parse_bg_layer(&layer)
            };

            bg.update_from_layer(parsed_layer);
            println!("BG after layer {}: {:?}", i, bg);
        }

        bg
    }

    fn update_from_layer(&mut self, layer: BackgroundLayer) {
        if let Some(image) = layer.image {
            self.image = Some(image);
        }

        if let Some(color) = layer.color {
            self.color = color;
        }
    }

    fn parse_bg_layer(cvs: &[ComponentValue]) -> BackgroundLayer {
        BackgroundLayer {
            image: None,
            color: None,
        }
    }

    fn parse_final_bg_layer(cvs: &[ComponentValue]) -> BackgroundLayer {
        println!("Final BG Layer CVs: {:?}", cvs);

        BackgroundLayer {
            image: None,
            color: Some(Color::from_cvs(cvs).unwrap_or(Color::transparent())),
        }
    }
}
