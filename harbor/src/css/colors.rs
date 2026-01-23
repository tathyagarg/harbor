use crate::{
    css::{
        parser::{ComponentValue, Function},
        properties::CSSParseable,
        tokenize::{CSSToken, HashToken},
    },
    infra::InputStream,
};

/// Name              Light Mode   Dark Mode
/// ----------------------------------------
/// AccentColor       #2563EB      #60A5FA
/// AccentColorText   #FFFFFF      #0A0A0A
/// ActiveText        #DC2626      #F87171
/// ButtonBorder      #D1D5DB      #374151
/// ButtonFace        #F3F4F6      #1F2937
/// ButtonText        #111827      #E5E7EB
/// Canvas            #FFFFFF      #0B0F14
/// CanvasText        #111827      #E5E7EB
/// Field             #FFFFFF      #111827
/// FieldText         #111827      #F9FAFB
/// GrayText          #6B7280      #9CA3AF
/// Highlight         #DBEAFE      #1E3A8A
/// HighlightText     #1E3A8A      #DBEAFE
/// LinkText          #2563EB      #60A5FA
/// Mark              #FEF3C7      #78350F
/// MarkText          #92400E      #FDE68A
/// SelectedItem      #BFDBFE      #1D4ED8
/// SelectedItemText  #1E3A8A      #DBEAFE
/// VisitedText       #7C3AED      #C4B5FD
///
///
/// AccentColor
pub const ACCENT_COLOR_LIGHT: &str = "#2563EB";
pub const ACCENT_COLOR_DARK: &str = "#60A5FA";

/// AccentColorText
pub const ACCENT_COLOR_TEXT_DARK: &str = "#FFFFFF";
pub const ACCENT_COLOR_TEXT_LIGHT: &str = "#0A0A0A";

/// ActiveText
pub const ACTIVE_TEXT_LIGHT: &str = "#DC2626";
pub const ACTIVE_TEXT_DARK: &str = "#F87171";

/// ButtonBorder
pub const BUTTON_BORDER_LIGHT: &str = "#D1D5DB";
pub const BUTTON_BORDER_DARK: &str = "#374151";

/// ButtonFace
pub const BUTTON_FACE_LIGHT: &str = "#F3F4F6";
pub const BUTTON_FACE_DARK: &str = "#1F2937";

/// ButtonText
pub const BUTTON_TEXT_LIGHT: &str = "#111827";
pub const BUTTON_TEXT_DARK: &str = "#E5E7EB";

/// Canvas
pub const CANVAS_LIGHT: &str = "#FFFFFF";
pub const CANVAS_DARK: &str = "#0B0F14";

/// CanvasText
pub const CANVAS_TEXT_LIGHT: &str = "#111827";
pub const CANVAS_TEXT_DARK: &str = "#E5E7EB";

/// Field
pub const FIELD_LIGHT: &str = "#FFFFFF";
pub const FIELD_DARK: &str = "#111827";

/// FieldText
pub const FIELD_TEXT_LIGHT: &str = "#111827";
pub const FIELD_TEXT_DARK: &str = "#F9FAFB";

/// GrayText
pub const GRAY_TEXT_LIGHT: &str = "#6B7280";
pub const GRAY_TEXT_DARK: &str = "#9CA3AF";

/// Highlight
pub const HIGHLIGHT_LIGHT: &str = "#DBEAFE";
pub const HIGHLIGHT_DARK: &str = "#1E3A8A";

/// HighlightText
pub const HIGHLIGHT_TEXT_LIGHT: &str = "#1E3A8A";
pub const HIGHLIGHT_TEXT_DARK: &str = "#DBEAFE";

/// LinkText
pub const LINK_TEXT_LIGHT: &str = "#2563EB";
pub const LINK_TEXT_DARK: &str = "#60A5FA";

/// Mark
pub const MARK_LIGHT: &str = "#FEF3C7";
pub const MARK_DARK: &str = "#78350F";

/// MarkText
pub const MARK_TEXT_LIGHT: &str = "#92400E";
pub const MARK_TEXT_DARK: &str = "#FDE68A";

/// SelectedItem
pub const SELECTED_ITEM_LIGHT: &str = "#BFDBFE";
pub const SELECTED_ITEM_DARK: &str = "#1D4ED8";

/// SelectedItemText
pub const SELECTED_ITEM_TEXT_LIGHT: &str = "#1E3A8A";
pub const SELECTED_ITEM_TEXT_DARK: &str = "#DBEAFE";

/// VisitedText
pub const VISITED_TEXT_LIGHT: &str = "#7C3AED";
pub const VISITED_TEXT_DARK: &str = "#C4B5FD";

pub fn is_system_color(name: &str) -> bool {
    matches!(
        name,
        "AccentColor"
            | "AccentColorText"
            | "ActiveText"
            | "ButtonBorder"
            | "ButtonFace"
            | "ButtonText"
            | "Canvas"
            | "CanvasText"
            | "Field"
            | "FieldText"
            | "GrayText"
            | "Highlight"
            | "HighlightText"
            | "LinkText"
            | "Mark"
            | "MarkText"
            | "SelectedItem"
            | "SelectedItemText"
            | "VisitedText"
    )
}

pub fn get_system_color(name: &str, dark_mode: bool) -> Option<&'static str> {
    match (name, dark_mode) {
        ("AccentColor", false) => Some(ACCENT_COLOR_LIGHT),
        ("AccentColor", true) => Some(ACCENT_COLOR_DARK),
        ("AccentColorText", false) => Some(ACCENT_COLOR_TEXT_LIGHT),
        ("AccentColorText", true) => Some(ACCENT_COLOR_TEXT_DARK),
        ("ActiveText", false) => Some(ACTIVE_TEXT_LIGHT),
        ("ActiveText", true) => Some(ACTIVE_TEXT_DARK),
        ("ButtonBorder", false) => Some(BUTTON_BORDER_LIGHT),
        ("ButtonBorder", true) => Some(BUTTON_BORDER_DARK),
        ("ButtonFace", false) => Some(BUTTON_FACE_LIGHT),
        ("ButtonFace", true) => Some(BUTTON_FACE_DARK),
        ("ButtonText", false) => Some(BUTTON_TEXT_LIGHT),
        ("ButtonText", true) => Some(BUTTON_TEXT_DARK),
        ("Canvas", false) => Some(CANVAS_LIGHT),
        ("Canvas", true) => Some(CANVAS_DARK),
        ("CanvasText", false) => Some(CANVAS_TEXT_LIGHT),
        ("CanvasText", true) => Some(CANVAS_TEXT_DARK),
        ("Field", false) => Some(FIELD_LIGHT),
        ("Field", true) => Some(FIELD_DARK),
        ("FieldText", false) => Some(FIELD_TEXT_LIGHT),
        ("FieldText", true) => Some(FIELD_TEXT_DARK),
        ("GrayText", false) => Some(GRAY_TEXT_LIGHT),
        ("GrayText", true) => Some(GRAY_TEXT_DARK),
        ("Highlight", false) => Some(HIGHLIGHT_LIGHT),
        ("Highlight", true) => Some(HIGHLIGHT_DARK),
        ("HighlightText", false) => Some(HIGHLIGHT_TEXT_LIGHT),
        ("HighlightText", true) => Some(HIGHLIGHT_TEXT_DARK),
        ("LinkText", false) => Some(LINK_TEXT_LIGHT),
        ("LinkText", true) => Some(LINK_TEXT_DARK),
        ("Mark", false) => Some(MARK_LIGHT),
        ("Mark", true) => Some(MARK_DARK),
        ("MarkText", false) => Some(MARK_TEXT_LIGHT),
        ("MarkText", true) => Some(MARK_TEXT_DARK),
        ("SelectedItem", false) => Some(SELECTED_ITEM_LIGHT),
        ("SelectedItem", true) => Some(SELECTED_ITEM_DARK),
        ("SelectedItemText", false) => Some(SELECTED_ITEM_TEXT_LIGHT),
        ("SelectedItemText", true) => Some(SELECTED_ITEM_TEXT_DARK),
        ("VisitedText", false) => Some(VISITED_TEXT_LIGHT),
        ("VisitedText", true) => Some(VISITED_TEXT_DARK),
        _ => None,
    }
}

/* Named Colors */
pub fn get_named_color(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "aliceblue" => Some("#f0f8ff"),
        "antiquewhite" => Some("#faebd7"),
        "aqua" => Some("#00ffff"),
        "aquamarine" => Some("#7fffd4"),
        "azure" => Some("#f0ffff"),
        "beige" => Some("#f5f5dc"),
        "bisque" => Some("#ffe4c4"),
        "black" => Some("#000000"),
        "blanchedalmond" => Some("#ffebcd"),
        "blue" => Some("#0000ff"),
        "blueviolet" => Some("#8a2be2"),
        "brown" => Some("#a52a2a"),
        "burlywood" => Some("#deb887"),
        "cadetblue" => Some("#5f9ea0"),
        "chartreuse" => Some("#7fff00"),
        "chocolate" => Some("#d2691e"),
        "coral" => Some("#ff7f50"),
        "cornflowerblue" => Some("#6495ed"),
        "cornsilk" => Some("#fff8dc"),
        "crimson" => Some("#dc143c"),
        "cyan" => Some("#00ffff"),
        "darkblue" => Some("#00008b"),
        "darkcyan" => Some("#008b8b"),
        "darkgoldenrod" => Some("#b8860b"),
        "darkgray" => Some("#a9a9a9"),
        "darkgreen" => Some("#006400"),
        "darkgrey" => Some("#a9a9a9"),
        "darkkhaki" => Some("#bdb76b"),
        "darkmagenta" => Some("#8b008b"),
        "darkolivegreen" => Some("#556b2f"),
        "darkorange" => Some("#ff8c00"),
        "darkorchid" => Some("#9932cc"),
        "darkred" => Some("#8b0000"),
        "darksalmon" => Some("#e9967a"),
        "darkseagreen" => Some("#8fbc8f"),
        "darkslateblue" => Some("#483d8b"),
        "darkslategray" => Some("#2f4f4f"),
        "darkslategrey" => Some("#2f4f4f"),
        "darkturquoise" => Some("#00ced1"),
        "darkviolet" => Some("#9400d3"),
        "deeppink" => Some("#ff1493"),
        "deepskyblue" => Some("#00bfff"),
        "dimgray" => Some("#696969"),
        "dimgrey" => Some("#696969"),
        "dodgerblue" => Some("#1e90ff"),
        "firebrick" => Some("#b22222"),
        "floralwhite" => Some("#fffaf0"),
        "forestgreen" => Some("#228b22"),
        "fuchsia" => Some("#ff00ff"),
        "gainsboro" => Some("#dcdcdc"),
        "ghostwhite" => Some("#f8f8ff"),
        "gold" => Some("#ffd700"),
        "goldenrod" => Some("#daa520"),
        "gray" => Some("#808080"),
        "green" => Some("#008000"),
        "greenyellow" => Some("#adff2f"),
        "grey" => Some("#808080"),
        "honeydew" => Some("#f0fff0"),
        "hotpink" => Some("#ff69b4"),
        "indianred" => Some("#cd5c5c"),
        "indigo" => Some("#4b0082"),
        "ivory" => Some("#fffff0"),
        "khaki" => Some("#f0e68c"),
        "lavender" => Some("#e6e6fa"),
        "lavenderblush" => Some("#fff0f5"),
        "lawngreen" => Some("#7cfc00"),
        "lemonchiffon" => Some("#fffacd"),
        "lightblue" => Some("#add8e6"),
        "lightcoral" => Some("#f08080"),
        "lightcyan" => Some("#e0ffff"),
        "lightgoldenrodyellow" => Some("#fafad2"),
        "lightgray" => Some("#d3d3d3"),
        "lightgreen" => Some("#90ee90"),
        "lightgrey" => Some("#d3d3d3"),
        "lightpink" => Some("#ffb6c1"),
        "lightsalmon" => Some("#ffa07a"),
        "lightseagreen" => Some("#20b2aa"),
        "lightskyblue" => Some("#87cefa"),
        "lightslategray" => Some("#778899"),
        "lightslategrey" => Some("#778899"),
        "lightsteelblue" => Some("#b0c4de"),
        "lightyellow" => Some("#ffffe0"),
        "lime" => Some("#00ff00"),
        "limegreen" => Some("#32cd32"),
        "linen" => Some("#faf0e6"),
        "magenta" => Some("#ff00ff"),
        "maroon" => Some("#800000"),
        "mediumaquamarine" => Some("#66cdaa"),
        "mediumblue" => Some("#0000cd"),
        "mediumorchid" => Some("#ba55d3"),
        "mediumpurple" => Some("#9370db"),
        "mediumseagreen" => Some("#3cb371"),
        "mediumslateblue" => Some("#7b68ee"),
        "mediumspringgreen" => Some("#00fa9a"),
        "mediumturquoise" => Some("#48d1cc"),
        "mediumvioletred" => Some("#c71585"),
        "midnightblue" => Some("#191970"),
        "mintcream" => Some("#f5fffa"),
        "mistyrose" => Some("#ffe4e1"),
        "moccasin" => Some("#ffe4b5"),
        "navajowhite" => Some("#ffdead"),
        "navy" => Some("#000080"),
        "oldlace" => Some("#fdf5e6"),
        "olive" => Some("#808000"),
        "olivedrab" => Some("#6b8e23"),
        "orange" => Some("#ffa500"),
        "orangered" => Some("#ff4500"),
        "orchid" => Some("#da70d6"),
        "palegoldenrod" => Some("#eee8aa"),
        "palegreen" => Some("#98fb98"),
        "paleturquoise" => Some("#afeeee"),
        "palevioletred" => Some("#db7093"),
        "papayawhip" => Some("#ffefd5"),
        "peachpuff" => Some("#ffdab9"),
        "peru" => Some("#cd853f"),
        "pink" => Some("#ffc0cb"),
        "plum" => Some("#dda0dd"),
        "powderblue" => Some("#b0e0e6"),
        "purple" => Some("#800080"),
        "rebeccapurple" => Some("#663399"),
        "red" => Some("#ff0000"),
        "rosybrown" => Some("#bc8f8f"),
        "royalblue" => Some("#4169e1"),
        "saddlebrown" => Some("#8b4513"),
        "salmon" => Some("#fa8072"),
        "sandybrown" => Some("#f4a460"),
        "seagreen" => Some("#2e8b57"),
        "seashell" => Some("#fff5ee"),
        "sienna" => Some("#a0522d"),
        "silver" => Some("#c0c0c0"),
        "skyblue" => Some("#87ceeb"),
        "slateblue" => Some("#6a5acd"),
        "slategray" => Some("#708090"),
        "slategrey" => Some("#708090"),
        "snow" => Some("#fffafa"),
        "springgreen" => Some("#00ff7f"),
        "steelblue" => Some("#4682b4"),
        "tan" => Some("#d2b48c"),
        "teal" => Some("#008080"),
        "thistle" => Some("#d8bfd8"),
        "tomato" => Some("#ff6347"),
        "turquoise" => Some("#40e0d0"),
        "violet" => Some("#ee82ee"),
        "wheat" => Some("#f5deb3"),
        "white" => Some("#ffffff"),
        "whitesmoke" => Some("#f5f5f5"),
        "yellow" => Some("#ffff00"),
        "yellowgreen" => Some("#9acd32"),

        // Best color ever
        "tattu" => Some("#1E90FF"),

        _ => None,
    }
}

pub fn is_color(token: &ComponentValue) -> bool {
    match token {
        ComponentValue::Token(CSSToken::Ident(name))
        // currentColor, transparent, <system-color>
            if name == "currentColor" || name == "transparent" || is_system_color(name) =>
        {
            true
        }
        // <named-color>
        ComponentValue::Token(CSSToken::Ident(name)) if get_named_color(name).is_some() => true,
        // <hex-color>
        ComponentValue::Token(CSSToken::Hash(HashToken{ value: val, .. })) => match val.len() {
            // #RRGGBB, #RGB, #RRGGBBAA, #RGBA
            3 | 6 | 4 | 8 => val.chars().all(|c| c.is_ascii_hexdigit()),
            _ => false,
        },
        // <color-function>
        ComponentValue::Function(Function(name, ..)) if is_color_function(name) =>
        {
            true
        }
        _ => false,
    }
}

pub fn is_color_function(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch" | "color"
    )
}

pub fn hex_to_rgb(hex: &str) -> UsedColor {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        3 => (
            u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0),
            100.0,
        ),
        4 => (
            u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap_or(0) as f32 * 100.0 / 255.0,
        ),
        6 => (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
            100.0,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(0) as f32 * 100.0 / 255.0,
        ),
        _ => (0, 0, 0, 100.0),
    };
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a / 100.0,
    ]
}

mod functions {
    use crate::css::{
        colors::UsedColor,
        parser::{ComponentValue, Function},
        tokenize::CSSToken,
    };

    pub fn from_rgb(func: &Function) -> Option<UsedColor> {
        if func.1.len() < 3 {
            return None;
        }

        let mut components = [0.0, 0.0, 0.0, 1.0];
        let mut index = 0;

        for cv in &func.1 {
            if index >= 4 {
                break;
            }

            match cv {
                ComponentValue::Token(CSSToken::Number { value, .. }) => {
                    if index == 3 {
                        components[index] = *value as f32;
                    } else {
                        components[index] = (*value as f32) / 255.0;
                    }
                    index += 1;
                }
                ComponentValue::Token(CSSToken::Percentage(perc)) => {
                    components[index] = (*perc as f32) / 100.0;
                    index += 1;
                }
                _ => {}
            }
        }

        Some(components)
    }
}

/// TODO: Make parse match spec
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Named(String),
    SystemNamed(String),
    Hex(String),
    Function(Function),
}

impl Default for Color {
    fn default() -> Self {
        Color::Named(String::from("black"))
    }
}

impl Color {
    pub fn transparent() -> Self {
        Color::Hex(String::from("#00000000"))
    }

    pub fn used(&self) -> [f32; 4] {
        match self {
            Color::Named(name) => {
                if let Some(hex) = get_named_color(name) {
                    hex_to_rgb(hex)
                } else {
                    [0.0, 0.0, 0.0, 0.0]
                }
            }
            Color::SystemNamed(name) => {
                if let Some(hex) = get_system_color(name, false) {
                    hex_to_rgb(hex)
                } else {
                    [0.0, 0.0, 0.0, 0.0]
                }
            }
            Color::Hex(hex) => hex_to_rgb(hex),
            Color::Function(func) => match func.0.to_lowercase().as_str() {
                "rgb" | "rgba" => functions::from_rgb(func).unwrap_or([0.0, 0.0, 0.0, 1.0]),
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        if get_named_color(name).is_some() {
            Some(Color::Named(name.to_string()))
        } else {
            None
        }
    }
}

impl CSSParseable for Color {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self> {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(name))
                    if get_named_color(&name).is_some() =>
                {
                    Some(Color::Named(name.clone()))
                }
                ComponentValue::Token(CSSToken::Ident(name)) if is_system_color(&name) => {
                    Some(Color::SystemNamed(name.clone()))
                }
                ComponentValue::Token(CSSToken::Hash(HashToken { value: val, .. })) => {
                    Some(Color::Hex(val.clone()))
                }
                ComponentValue::Function(func) if is_color_function(&func.0) => {
                    Some(Color::Function(func.clone()))
                }
                _ => None,
            }
        } else {
            cvs.reconsume();
            None
        }
    }
}

pub type UsedColor = [f32; 4];
