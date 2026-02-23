const VALID_RELS: [&str; 13] = [
    "alternate",
    "dns-prefetch",
    "expect",
    "icon",
    "manifest",
    "modulepreload",
    "next",
    "pingback",
    "preconnect",
    "prefetch",
    "preload",
    "search",
    "stylesheet",
];

pub struct LinkElement {
    pub disabled: bool,
    pub href: String,
    pub rel: String,

    // NOTE: This is readonly
    pub _rel_list: Vec<String>,

    pub media: String,
    pub hreflang: String,
    pub type_: String,
}

impl LinkElement {
    pub fn new() -> Self {
        Self {
            disabled: false,
            href: String::new(),
            rel: String::new(),
            _rel_list: Vec::new(),
            media: String::new(),
            hreflang: String::new(),
            type_: String::new(),
        }
    }

    pub fn rel_list(&self) -> Vec<String> {
        self._rel_list.clone()
    }

    pub fn verify_rel(&self) -> bool {
        for rel in self.rel.split_whitespace() {
            if !VALID_RELS.contains(&rel) {
                return false;
            }
        }

        return true;
    }

    pub fn verify(&self) -> bool {
        if self.rel.is_empty() || self.href.is_empty() {
            return false;
        }

        return self.verify_rel();
    }
}
