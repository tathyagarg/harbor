use std::{cell::RefCell, rc::Rc};

use crate::{
    css::{
        r#box::BoxType,
        cssom::ComputedStyle,
        parser::{ComponentValue, Function},
        tokenize::CSSToken,
    },
    html5::dom::Element,
    infra::InputStream,
};

macro_rules! prop_imports {
    () => {
        #[allow(unused_imports)]
        use crate::{
            css::{
                colors::Color,
                cssom::ComputedStyle,
                parser::ComponentValue,
                properties::{CSSParseable, Resolvable},
                tokenize::CSSToken,
            },
            globals::DEFAULT_FONT_SIZE,
            html5::dom::Element,
            infra::InputStream,
        };

        #[allow(unused_imports)]
        use std::{cell::RefCell, rc::Rc};
    };
}
pub(super) use prop_imports;

pub mod background;
pub mod cursor;
pub mod font;
pub mod image;
pub mod length_percentage;
pub mod margin;
pub mod position;
pub mod width;

pub use background::*;
pub use cursor::*;
pub use font::*;
pub use image::*;
pub use length_percentage::*;
pub use margin::*;
pub use position::*;
pub use width::*;

pub trait CSSParseable {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized;
}

pub trait Resolvable<T> {
    fn resolve(&mut self, parents: &Vec<Rc<RefCell<Element>>>) -> T;

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        current: &ComputedStyle,
        viewport_size: (f64, f64),
    ) -> T;

    fn resolved(&self) -> T;
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

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub enum Display {
    #[default]
    Inline,
    Block,
    ListItem,
    None,
}

impl CSSParseable for Display {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "inline" => return Some(Display::Inline),
                    "block" => return Some(Display::Block),
                    "list-item" => return Some(Display::ListItem),
                    "none" => return Some(Display::None),
                    _ => {
                        todo!("Handle more display values")
                    }
                },
                _ => {}
            }
        }

        cvs.reconsume();
        None
    }
}

impl Display {
    pub fn to_box_type(&self) -> BoxType {
        match self {
            Display::Inline => BoxType::Inline,
            Display::Block => BoxType::Block,
            Display::ListItem => BoxType::ListItem,
            Display::None => BoxType::None,
        }
    }
}
