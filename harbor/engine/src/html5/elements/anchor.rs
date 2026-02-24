use crate::html5::{dom::Element, elements::HyperlinkUtils};

pub struct AnchorElement {
    pub target: Option<String>,
    pub download: Option<String>,
    pub ping: Option<String>,
    pub rel: Option<String>,
    pub rel_list: Vec<String>,
    pub hreflang: Option<String>,
    pub type_attr: Option<String>,
    pub text: String,
    pub referrer_policy: Option<String>,

    pub hyperlink_utils: Option<HyperlinkUtils>,
}

impl AnchorElement {
    pub fn from_element(elem: &Element) -> Self {
        let hyperlink_utils = if let Some(href) = elem.get_attribute("href") {
            HyperlinkUtils::new(&href)
        } else {
            None
        };

        Self {
            target: elem.get_attribute("target").map(|s| s.to_string()),
            download: elem.get_attribute("download").map(|s| s.to_string()),
            ping: elem.get_attribute("ping").map(|s| s.to_string()),
            rel: elem.get_attribute("rel").map(|s| s.to_string()),
            rel_list: elem
                .get_attribute("rel")
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            hreflang: elem.get_attribute("hreflang").map(|s| s.to_string()),
            type_attr: elem.get_attribute("type").map(|s| s.to_string()),
            text: elem.text_content().unwrap_or_default(),
            referrer_policy: elem.get_attribute("referrerpolicy").map(|s| s.to_string()),
            hyperlink_utils,
        }
    }

    pub fn verify(&self) -> bool {
        if self.hyperlink_utils.is_none()
            && (self.target.is_some()
                || self.download.is_some()
                || self.ping.is_some()
                || self.rel.is_some()
                || self.hreflang.is_some()
                || self.type_attr.is_some()
                || self.referrer_policy.is_some())
        {
            return false;
        }

        true
    }
}
