use crate::font::ttc::TTCData;
use crate::font::{self};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

type ResPath = fn() -> PathBuf;

pub const ROOT_PATH: ResPath = || {
    let res = std::path::PathBuf::from(std::env::current_exe().unwrap().parent().unwrap());
    if std::env::var("HARBOR_DEV").is_ok_and(|v| v == "super_secret_password") {
        return std::path::PathBuf::from(res.parent().unwrap().parent().unwrap().parent().unwrap());
    }

    res
};

pub const RES_PATH: ResPath = || ROOT_PATH().join("res");

macro_rules! page_path {
    ($res:expr) => {
        RES_PATH().join("pages").join($res)
    };
}

pub const UA_CSS_PATH: ResPath = || RES_PATH().join("css").join("ua.css");

pub const DEFAULT_FONT_FAMILY: &str = "sans-serif";
pub const DEFAULT_FONT_WEIGHT: u16 = 400;
pub const DEFAULT_FONT_STYLE_ITALIC: bool = false;
pub const DEFAULT_FONT_SIZE: f64 = 16.0;
pub const DEFAULT_LINE_HEIGHT: f64 = 1.2;

pub const INITIAL_WINDOW_WIDTH: u32 = 800;
pub const INITIAL_WINDOW_HEIGHT: u32 = 600;

pub const MINIMUM_WINDOW_WIDTH: u32 = 400;
pub const MINIMUM_WINDOW_HEIGHT: u32 = 300;

pub const NEW_TAB_URL: &str = "harbor:new-tab";
pub const NEW_TAB: &str = "new-tab";
pub const NEW_TAB_PAGE_PATH: ResPath = || page_path!("tab.html");

pub const NO_CONNECTION_URL: &str = "harbor:no-connection";
pub const NO_CONNECTION: &str = "no-connection";
pub const NO_CONNECTION_PAGE_PATH: ResPath = || page_path!("no_connection.html");

pub const ERROR_URL: &str = "harbor:error";
pub const ERROR: &str = "error";
pub const ERROR_PAGE_PATH: ResPath = || page_path!("error.html");

pub const FONTS_PATH: ResPath = || RES_PATH().join("fonts");

pub const TAB_BAR_HORIZONTAL: bool = false;

// TODO: Make this configurable
pub const TABS_BAR_OFFSET: fn(f64, f64) -> (f64, f64) = |window_width, window_height| {
    if TAB_BAR_HORIZONTAL {
        (0.0, (window_height * 0.05).min(50.0))
    } else {
        (window_width * 0.20, 0.0)
    }
};

pub const ADDRESS_BAR_OFFSET: fn(f64, f64) -> (f64, f64) = |_window_width, window_height| {
    if TAB_BAR_HORIZONTAL {
        (0.0, (window_height * 0.05).min(50.0))
    } else {
        (0.0, (window_height * 0.05).min(50.0))
    }
};

// pub const ADDRESS_BAR_ADDRESS_OFFSET: fn(f64, f64) -> (f64, f64) =
//     |window_width, _window_height| (window_width * 0.1, 0.0);
pub const ADDRESS_BAR_ADDRESS_OFFSET: fn(f64, f64) -> (f64, f64) =
    |window_width, _window_height| {
        if TAB_BAR_HORIZONTAL {
            (window_width * 0.1, 0.0)
        } else {
            (window_width * 0.1, 0.0)
        }
    };

pub const TOOLBAR_OFFSET: fn(f64, f64) -> (f64, f64) = |window_width, window_height| {
    let tabs_bar_offset = TABS_BAR_OFFSET(window_width, window_height);
    let address_bar_offset = ADDRESS_BAR_OFFSET(window_width, window_height);

    (
        tabs_bar_offset.0 + address_bar_offset.0,
        tabs_bar_offset.1 + address_bar_offset.1,
    )
};

// pub const TAB_WIDTH: fn(f64, usize) -> f64 =
//     |window_width, num_tabs| window_width / (4.max(num_tabs) as f64);
pub const TAB_WIDTH: fn(f64, usize) -> f64 = |window_width, num_tabs| {
    if TAB_BAR_HORIZONTAL {
        window_width / (4.max(num_tabs) as f64)
    } else {
        TABS_BAR_OFFSET(window_width, 0.0).0
    }
};

pub const TAB_HEIGHT: fn(f64, f64) -> f64 = |window_width, window_height| {
    if TAB_BAR_HORIZONTAL {
        TABS_BAR_OFFSET(window_width, window_height).1
    } else {
        window_height * 0.05
    }
};

pub static FONTS: LazyLock<HashMap<String, Arc<TTCData>>> = LazyLock::new(|| {
    let arial = Arc::new(font::parse_ttc(
        fs::read(FONTS_PATH().join("Arial.ttc")).unwrap().as_slice(),
    ));

    let verdana = Arc::new(font::parse_ttc(
        fs::read(FONTS_PATH().join("Verdana.ttc"))
            .unwrap()
            .as_slice(),
    ));

    let tahoma = Arc::new(TTCData::new(vec![font::parse_ttf(
        fs::read(FONTS_PATH().join("Tahoma.ttf"))
            .unwrap()
            .as_slice(),
    )]));

    let trebuchet_ms = Arc::new(font::parse_ttc(
        fs::read(FONTS_PATH().join("TrebuchetMS.ttc"))
            .unwrap()
            .as_slice(),
    ));

    let georgia = Arc::new(font::parse_ttc(
        fs::read(FONTS_PATH().join("Georgia.ttc"))
            .unwrap()
            .as_slice(),
    ));

    let garamond = Arc::new(font::parse_ttc(
        fs::read(FONTS_PATH().join("Garamond.ttc"))
            .unwrap()
            .as_slice(),
    ));

    let courier_prime = Arc::new(font::parse_ttc(
        fs::read(FONTS_PATH().join("CourierPrime.ttc"))
            .unwrap()
            .as_slice(),
    ));

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
