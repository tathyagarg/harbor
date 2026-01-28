use crate::font::ttc::TTCData;
use crate::font::{self};

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub static FONTS: LazyLock<HashMap<String, Arc<TTCData>>> = LazyLock::new(|| {
    let fira_code_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/FiraCode.ttf"));
    let fira_code = Arc::new(TTCData::new(vec![fira_code_ttf]));

    let times = Arc::new(font::parse_ttc(include_bytes!(
        "../../assets/fonts/Times.ttc"
    )));

    let sfns_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/SFNS.ttf"));
    let sfns = Arc::new(TTCData::new(vec![sfns_ttf]));

    let andika = Arc::new(font::parse_ttc(include_bytes!(
        "../../assets/fonts/Andika.ttc"
    )));

    let mut map = HashMap::new();
    map.insert("FiraCode".to_string(), fira_code);
    map.insert("Times New Roman".to_string(), times);
    map.insert("SFNS".to_string(), sfns.clone());
    map.insert("Andika".to_string(), andika);

    map.insert("sans-serif".to_string(), sfns);

    map
});
