use crate::font::ttc::{CompleteTTCData, TTCData};
use crate::font::{self};

use std::collections::HashMap;
use std::sync::LazyLock;

pub static FONTS: LazyLock<HashMap<String, CompleteTTCData>> = LazyLock::new(|| {
    let fira_code_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/FiraCode.ttf"));
    let fira_code = TTCData::new(vec![fira_code_ttf]).complete();

    let times = font::parse_ttc(include_bytes!("../../assets/fonts/Times.ttc")).complete();

    let sfns_ttf = font::parse_ttf(include_bytes!("../../assets/fonts/SFNS.ttf"));
    let sfns = TTCData::new(vec![sfns_ttf]).complete();

    let andika = font::parse_ttc(include_bytes!("../../assets/fonts/Andika.ttc")).complete();

    let mut map = HashMap::new();
    map.insert("FiraCode".to_string(), fira_code);
    map.insert("Times New Roman".to_string(), times);
    map.insert("SFNS".to_string(), sfns.clone());
    map.insert("Andika".to_string(), andika);

    map.insert("sans-serif".to_string(), sfns);

    map
});
