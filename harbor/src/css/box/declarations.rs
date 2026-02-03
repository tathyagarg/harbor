use std::{cell::RefCell, rc::Rc};

use crate::{
    css::{
        colors::Color,
        cssom::{CSSDeclaration, ComputedStyle},
        properties::{
            Background, Bottom, CSSParseable, Display, Font, FontFamily, FontSize, FontStyle,
            FontWeight, Image, Left, LineHeight, Margin, MarginValue, Origin, Position,
            PositionValue, RepeatStyle, Resolvable, Right, Top, WidthValue,
        },
    },
    html5::dom::Element,
    infra::InputStream,
};

fn handle_background(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    let mut stream = InputStream::new(&declaration.value);

    let bg = Background::from_cv(&mut stream);
    if let Some(bg) = bg {
        style.background = bg;
    }
}

fn handle_background_property(declaration: &CSSDeclaration, style: &mut ComputedStyle) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "background-color" => {
            let color = Color::from_cv(&mut stream);
            if let Some(color) = color {
                style.background.set_color(color);
            }
        }
        "background-image" => {
            let bg_image = Image::parse_multiple_images(&mut stream);
            style.background.set_images(bg_image);
        }
        "background-repeat" => {
            let repeat = RepeatStyle::parse_multiple_repeat_styles(&mut stream);
            style.background.set_repeat_styles(repeat);
        }
        "background-position" => {
            let position = PositionValue::parse_multiple_positions(&mut stream);
            style.background.set_positions(position);
        }
        "background-origin" => {
            let origin = Origin::parse_multiple_origins(&mut stream);
            style.background.set_origins(origin);
        }
        _ => {}
    }
}

fn handle_font(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
    viewport_size: (f64, f64),
) {
    let mut stream = InputStream::new(&declaration.value);

    let font = Font::from_cv(&mut stream);
    if let Some(mut font) = font {
        font.resolve_font_size(parents.unwrap_or(&vec![]));
        font.resolve_font_weight(parents.unwrap_or(&vec![]));
        font.resolve_line_height_curr(parents.unwrap_or(&vec![]), style, viewport_size);

        style.font = font;
    }
}

fn handle_font_property(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
    viewport_size: (f64, f64),
) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "font-family" => {
            let family = FontFamily::from_cv(&mut stream);
            if let Some(family) = family {
                style.font.set_family(family);
            }
        }
        "font-size" => {
            let size = FontSize::from_cv(&mut stream);
            if let Some(size) = size {
                style.font.set_size(size);
                style.font.resolve_font_size(parents.unwrap_or(&vec![]));

                let mut line_height = style.font.line_height();
                line_height.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);
                style.font.set_line_height(line_height);
            }
        }
        "font-weight" => {
            let weight = FontWeight::from_cv(&mut stream);
            if let Some(weight) = weight {
                style.font.set_weight(weight);
                style.font.resolve_font_weight(parents.unwrap_or(&vec![]));
            }
        }
        "line-height" => {
            let line_height = LineHeight::from_cv(&mut stream);
            if let Some(mut line_height) = line_height {
                line_height.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);

                style.font.set_line_height(line_height);
            }
        }
        "font-style" => {
            let font_style = FontStyle::from_cv(&mut stream);
            if let Some(font_style) = font_style {
                style.font.set_style(font_style);
            }
        }
        _ => {}
    }
}

fn handle_margin(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
    viewport_size: (f64, f64),
) {
    let mut stream = InputStream::new(&declaration.value);

    let margin = Margin::from_cv(&mut stream);
    if let Some(mut margin) = margin {
        // println!("Parsed margin: {:?}", margin);
        // println!("Style: {:#?}", style);
        margin.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);
        style.margin = margin;
    }
}

fn handle_margin_property(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
    viewport_size: (f64, f64),
) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "margin-top" => {
            let top = MarginValue::from_cv(&mut stream);
            if let Some(top) = top {
                let mut margin = style.margin.clone();
                margin.top = top;
                margin.resolve_top_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);

                style.margin = margin;
            }
        }
        "margin-right" => {
            let right = MarginValue::from_cv(&mut stream);
            if let Some(right) = right {
                let mut margin = style.margin.clone();
                margin.right = right;
                margin.resolve_right_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);

                style.margin = margin;
            }
        }
        "margin-bottom" => {
            let bottom = MarginValue::from_cv(&mut stream);
            if let Some(bottom) = bottom {
                let mut margin = style.margin.clone();
                margin.bottom = bottom;
                margin.resolve_bottom_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);

                style.margin = margin;
            }
        }
        "margin-left" => {
            let left = MarginValue::from_cv(&mut stream);
            if let Some(left) = left {
                let mut margin = style.margin.clone();
                margin.left = left;
                margin.resolve_left_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);

                style.margin = margin;
            }
        }
        _ => {}
    }
}

pub fn handle_declaration(
    declaration: &CSSDeclaration,
    style: &mut ComputedStyle,
    parents: Option<&Vec<Rc<RefCell<Element>>>>,
    viewport_size: (f64, f64),
) {
    let mut stream = InputStream::new(&declaration.value);

    match declaration.property_name.as_str() {
        "color" => {
            style.color = Color::from_cv(&mut stream).unwrap_or(Color::default());
        }
        "background" => {
            handle_background(declaration, style);
        }
        prop if prop.starts_with("background-") => {
            handle_background_property(declaration, style);
        }
        "font" => {
            handle_font(declaration, style, parents, viewport_size);
        }
        prop if prop.starts_with("font-") || prop == "line-height" => {
            handle_font_property(declaration, style, parents, viewport_size);
        }
        "width" => {
            let mut width = WidthValue::from_cv(&mut stream).unwrap_or_default();
            width.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);

            style.width = width;
        }
        "display" => {
            style.display = Display::from_cv(&mut stream).unwrap_or_default();
        }
        "margin" => {
            handle_margin(declaration, style, parents, viewport_size);
        }
        prop if prop.starts_with("margin-") => {
            handle_margin_property(declaration, style, parents, viewport_size);
        }
        "position" => {
            style.position = Position::from_cv(&mut stream).unwrap_or_default();
        }
        "top" => {
            let mut top = Top::from_cv(&mut stream).unwrap_or_default();
            top.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);
            style.top = top;
        }
        "left" => {
            let mut left = Left::from_cv(&mut stream).unwrap_or_default();
            left.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);
            style.left = left;
        }
        "right" => {
            let mut right = Right::from_cv(&mut stream).unwrap_or_default();
            right.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);
            style.right = right;
        }
        "bottom" => {
            let mut bottom = Bottom::from_cv(&mut stream).unwrap_or_default();
            bottom.resolve_with_curr(parents.unwrap_or(&vec![]), style, viewport_size);
            style.bottom = bottom;
        }
        _ => {
            // todo!(
            //     "Implement handling for property: {}",
            //     declaration.property_name
            // );
        }
    }
}
