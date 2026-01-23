use crate::font::ttc::{CompleteTTCData, TTCData};
use crate::font::{self};

use std::collections::HashMap;
use std::sync::LazyLock;

pub static FONTS: LazyLock<HashMap<String, CompleteTTCData>> = LazyLock::new(|| {
    let fira_code_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/FiraCode.ttf"));
    let fira_code = TTCData::new(vec![fira_code_ttf]);

    let times = font::parse_ttc(include_bytes!("../../assets/fonts/Times.ttc"));

    let sfns_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/SFNS.ttf"));
    let sfns = TTCData::new(vec![sfns_ttf]);

    let andika = font::parse_ttc(include_bytes!("../../assets/fonts/Andika.ttc"));

    let mut map = HashMap::new();
    map.insert("FiraCode".to_string(), fira_code.complete());
    map.insert("Times New Roman".to_string(), times.complete());
    map.insert("SFNS".to_string(), sfns.complete());
    map.insert("Andika".to_string(), andika.complete());

    map
});
