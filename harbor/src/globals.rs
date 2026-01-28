use crate::font::ttc::TTCData;
use crate::font::{self};

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub const DEFAULT_FONT_FAMILY: &str = "sans-serif";

pub static FONTS: LazyLock<HashMap<String, Arc<TTCData>>> = LazyLock::new(|| {
    let arial = Arc::new(font::parse_ttc(include_bytes!("../../res/fonts/Arial.ttc")));

    let verdana = Arc::new(font::parse_ttc(include_bytes!(
        "../../res/fonts/Verdana.ttc"
    )));

    let tahoma = Arc::new(TTCData::new(vec![font::parse_ttf(include_bytes!(
        "../../res/fonts/Tahoma.ttf"
    ))]));

    let trebuchet_ms = Arc::new(font::parse_ttc(include_bytes!(
        "../../res/fonts/TrebuchetMS.ttc"
    )));

    let georgia = Arc::new(font::parse_ttc(include_bytes!(
        "../../res/fonts/Georgia.ttc"
    )));

    let garamond = Arc::new(font::parse_ttc(include_bytes!(
        "../../res/fonts/Garamond.ttc"
    )));

    let courier_prime = Arc::new(font::parse_ttc(include_bytes!(
        "../../res/fonts/CourierPrime.ttc"
    )));

    let mut map: HashMap<String, Arc<TTCData>> = HashMap::new();

    // Sans-serif fonts
    map.insert("sans-serif".to_string(), arial.clone());
    map.insert("ui-sans-serif".to_string(), arial.clone());
    map.insert("Arial".to_string(), arial);

    map.insert("Verdana".to_string(), verdana);

    map.insert("Tahoma".to_string(), tahoma);

    map.insert("Trebuchet MS".to_string(), trebuchet_ms);

    // Serif fonts
    map.insert("serif".to_string(), georgia.clone());
    map.insert("ui-serif".to_string(), georgia.clone());
    map.insert("Georgia".to_string(), georgia);

    map.insert("Garamond".to_string(), garamond);

    // Monospace fonts
    map.insert("monospace".to_string(), courier_prime.clone());
    map.insert("ui-monospace".to_string(), courier_prime.clone());
    map.insert("Courier New".to_string(), courier_prime.clone());
    map.insert("Courier Prime".to_string(), courier_prime);

    map

    // let fira_code_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/FiraCode.ttf"));
    // let fira_code = Arc::new(TTCData::new(vec![fira_code_ttf]));

    // let times = Arc::new(font::parse_ttc(include_bytes!(
    //     "../../assets/fonts/Times.ttc"
    // )));

    // let sfns_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/SFNS.ttf"));
    // let sfns = Arc::new(TTCData::new(vec![sfns_ttf]));

    // let andika = Arc::new(font::parse_ttc(include_bytes!(
    //     "../../assets/fonts/Andika.ttc"
    // )));

    // let mut map = HashMap::new();
    // map.insert("FiraCode".to_string(), fira_code);
    // map.insert("Times New Roman".to_string(), times);
    // map.insert("SFNS".to_string(), sfns.clone());
    // map.insert("Andika".to_string(), andika);

    // map.insert("sans-serif".to_string(), sfns);

    // map
});
