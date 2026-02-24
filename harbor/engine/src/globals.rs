use crate::font::ttc::TTCData;
use crate::font::{self};

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub const FRAMES_IN_FLIGHT: usize = 3;

pub const DEFAULT_FONT_FAMILY: &str = "sans-serif";
pub const DEFAULT_FONT_WEIGHT: u16 = 400;
pub const DEFAULT_FONT_STYLE_ITALIC: bool = false;
pub const DEFAULT_FONT_SIZE: f64 = 16.0;
pub const DEFAULT_LINE_HEIGHT: f64 = 1.2;

pub const INITIAL_WINDOW_WIDTH: u32 = 800;
pub const INITIAL_WINDOW_HEIGHT: u32 = 600;

pub const MINIMUM_WINDOW_WIDTH: u32 = 400;
pub const MINIMUM_WINDOW_HEIGHT: u32 = 300;

// TODO: Make this configurable
pub const TABS_BAR_OFFSET: fn(f64, f64) -> (f64, f64) =
    |_window_width, window_height| (0.0, (window_height * 0.05).min(50.0));

pub const TAB_WIDTH: fn(f64, usize) -> f64 =
    |window_width, num_tabs| (window_width / (4.max(num_tabs) as f64));

pub static FONTS: LazyLock<HashMap<String, Arc<TTCData>>> = LazyLock::new(|| {
    let arial = Arc::new(font::parse_ttc(include_bytes!("../res/fonts/Arial.ttc")));

    let verdana = Arc::new(font::parse_ttc(include_bytes!("../res/fonts/Verdana.ttc")));

    let tahoma = Arc::new(TTCData::new(vec![font::parse_ttf(include_bytes!(
        "../res/fonts/Tahoma.ttf"
    ))]));

    let trebuchet_ms = Arc::new(font::parse_ttc(include_bytes!(
        "../res/fonts/TrebuchetMS.ttc"
    )));

    let georgia = Arc::new(font::parse_ttc(include_bytes!("../res/fonts/Georgia.ttc")));

    let garamond = Arc::new(font::parse_ttc(include_bytes!("../res/fonts/Garamond.ttc")));

    let courier_prime = Arc::new(font::parse_ttc(include_bytes!(
        "../res/fonts/CourierPrime.ttc"
    )));

    let mut map: HashMap<String, Arc<TTCData>> = HashMap::new();

    // Sans-serif fonts
    map.insert("sans-serif".to_string(), arial.clone());
    map.insert("ui-sans-serif".to_string(), arial.clone());
    map.insert("arial".to_string(), arial.clone());

    map.insert("verdana".to_string(), verdana);

    map.insert("tahoma".to_string(), tahoma);

    map.insert("trebuchet ms".to_string(), trebuchet_ms);

    // Serif fonts
    map.insert("serif".to_string(), georgia.clone());
    map.insert("ui-serif".to_string(), georgia.clone());
    map.insert("georgia".to_string(), georgia);

    map.insert("garamond".to_string(), garamond);

    // Monospace fonts
    map.insert("monospace".to_string(), courier_prime.clone());
    map.insert("ui-monospace".to_string(), courier_prime.clone());
    map.insert("courier new".to_string(), courier_prime.clone());
    map.insert("courier prime".to_string(), courier_prime);

    map.insert("system-ui".to_string(), arial);

    map
});
