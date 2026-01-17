use crate::font;
use crate::font::ttf::ParsedTableDirectory;

use std::collections::HashMap;
use std::sync::LazyLock;

pub static FONTS: LazyLock<HashMap<String, ParsedTableDirectory>> = LazyLock::new(|| {
    let fira_code = font::parse_ttf(include_bytes!("../../assets/fonts/FiraCode.ttf"));
    let times =
        &font::parse_ttc(include_bytes!("../../assets/fonts/Times.ttc")).table_directories[0];
    let sfns = font::parse_ttf(include_bytes!("../../assets/fonts/SFNS.ttf"));

    let mut map = HashMap::new();
    map.insert("FiraCode".to_string(), fira_code.complete());
    map.insert("Times New Roman".to_string(), times.complete());
    map.insert("SFNS".to_string(), sfns.complete());

    map
});
