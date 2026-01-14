use std::{
    cell::{Ref, RefCell},
    rc::Weak,
};

use crate::{
    css::parser::CSSToken,
    html5::dom::{Document, Element},
    http::url::{Serializable, URL},
};

pub struct CSSRule;

pub struct MediaList;

pub struct CSSStyleSheet {
    /// The type of the stylesheet (e.g., "text/css").
    pub _type: String,

    /// Specified when created. The absolute-URL string of the first request of the CSS style sheet
    /// or null if the CSS style sheet was embedded. Does not change during the lifetime of the CSS
    /// style sheet.
    _location: Option<String>,

    /// Specified when created. The CSS style sheet that is the parent of the CSS style sheet or null
    /// if there is no associated parent.
    _parent_style_sheet: Option<Box<CSSStyleSheet>>,

    /// Specified when created. The DOM node associated with the CSS style sheet or null if there is
    /// no associated DOM node.
    _owner_node: Option<Weak<RefCell<Element>>>,

    /// Specified when created. The CSS rule in the parent CSS style sheet that caused the inclusion
    /// of the CSS style sheet or null if there is no associated rule.
    _owner_rule: Option<CSSRule>,

    /// Specified when created. The MediaList object associated with the CSS style sheet.
    _media: MediaList,

    /// Specified when created. The title of the CSS style sheet, which can be the empty string.
    _title: String,

    /// Specified when created. Either set or unset. Unset by default.
    _alternate: bool,

    /// Either set or unset. Unset by default.
    _disabled: bool,

    /// The CSS rules associated with the CSS style sheet.
    _css_rules: Vec<CSSRule>,

    /// Specified when created. Either set or unset. If it is set, the API allows reading and modifying
    /// of the CSS rules.
    _origin_clean: bool,

    /// Specified when created. Either set or unset. Unset by default. Signifies whether this stylesheet
    /// was created by invoking the IDL-defined constructor.
    _constructed: bool,

    /// Either set or unset. Unset by default. If set, modification of the stylesheet’s rules is not
    /// allowed.
    _disallow_modifications: bool,

    /// Specified when created. The Document a constructed stylesheet is associated with.
    /// Null by default. Only non-null for stylesheets that have constructed flag set.
    _associated_document: Weak<RefCell<Document>>,

    /// The base URL to use when resolving relative URLs in the stylesheet. Null by default.
    /// Only non-null for stylesheets that have constructed flag set.
    _stylesheet_base_url: Option<String>,
}

trait StyleSheet {
    fn _type(&self) -> &String;
    fn href(&self) -> &Option<String>;

    /// Sounds like this should return a `Node`, but the spec says it returns an `Element`.
    /// > readonly attribute (Element or ProcessingInstruction)? ownerNode;
    fn owner_node(&self) -> &Option<Weak<RefCell<Element>>>;

    fn parent_style_sheet(&self) -> &Option<Box<CSSStyleSheet>>;

    fn title(&self) -> Option<&String>;

    fn media(&self) -> &MediaList;

    /// [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
    fn set_media(&mut self, media: MediaList);

    fn disabled(&self) -> bool;
    fn set_disabled(&mut self, disabled: bool);
}

impl StyleSheet for CSSStyleSheet {
    fn _type(&self) -> &String {
        &self._type
    }

    fn href(&self) -> &Option<String> {
        &self._location
    }

    fn owner_node(&self) -> &Option<Weak<RefCell<Element>>> {
        &self._owner_node
    }

    fn parent_style_sheet(&self) -> &Option<Box<CSSStyleSheet>> {
        &self._parent_style_sheet
    }

    fn title(&self) -> Option<&String> {
        if self._title.is_empty() {
            None
        } else {
            Some(&self._title)
        }
    }

    fn media(&self) -> &MediaList {
        &self._media
    }

    fn set_media(&mut self, media: MediaList) {
        todo!("Update underlying CSS rules to reflect media change");
        self._media = media;
    }

    fn disabled(&self) -> bool {
        self._disabled
    }

    fn set_disabled(&mut self, disabled: bool) {
        self._disabled = disabled;
    }
}

#[derive(Default)]
struct CSSStyleSheetInit {
    base_url: Option<String>,
    media: Option<MediaList>,
    disabled: Option<bool>,
}

trait CSSStyleSheetExt {
    fn new(options: Option<CSSStyleSheetInit>, document: Weak<RefCell<Document>>) -> Self;

    fn owner_rule(&self) -> &Option<CSSRule>;
    fn css_rules(&self) -> &Vec<CSSRule>;

    fn insert_rule(&mut self, rule: String, index: Option<usize>) -> Result<usize, String>;
    fn delete_rule(&mut self, index: usize) -> Result<(), String>;

    fn replace(&mut self, text: String) -> Result<(), String>;
}

impl CSSStyleSheetExt for CSSStyleSheet {
    fn new(options: Option<CSSStyleSheetInit>, document: Weak<RefCell<Document>>) -> Self {
        let init = options.unwrap_or_default();
        CSSStyleSheet {
            _type: "text/css".to_string(),
            _location: document
                .upgrade()
                .and_then(|doc| Some(doc.borrow().document_base_url().serialize())),
            _parent_style_sheet: None,
            _owner_node: None,
            _owner_rule: None,
            // TODO: Sort this one proper once
            _media: init.media.unwrap_or(MediaList),
            _title: String::new(),
            _alternate: false,
            _disabled: init.disabled.unwrap_or(false),
            _css_rules: Vec::new(),
            _origin_clean: true,
            _constructed: true,
            _disallow_modifications: false,
            _associated_document: document,
            _stylesheet_base_url: init.base_url,
        }
    }

    fn owner_rule(&self) -> &Option<CSSRule> {
        &self._owner_rule
    }

    fn css_rules(&self) -> &Vec<CSSRule> {
        &self._css_rules
    }

    fn insert_rule(&mut self, rule: String, index: Option<usize>) -> Result<usize, String> {
        // Placeholder implementation
        let idx = index.unwrap_or(self._css_rules.len());
        if idx > self._css_rules.len() {
            return Err("Index out of bounds".to_string());
        }
        self._css_rules.insert(idx, CSSRule);
        Ok(idx)
    }

    fn delete_rule(&mut self, index: usize) -> Result<(), String> {
        if index >= self._css_rules.len() {
            return Err("Index out of bounds".to_string());
        }
        self._css_rules.remove(index);
        Ok(())
    }

    fn replace(&mut self, text: String) -> Result<(), String> {
        // Placeholder implementation
        self._css_rules.clear();
        // In a real implementation, parse `text` and populate `_css_rules`
        Ok(())
    }
}

// impl CSSStyleSheet {
//     fn parse(
//         input: Vec<CSSToken>,
//         document: Weak<RefCell<Document>>,
//         location: Option<URL>,
//     ) -> Self {
//         let mut sheet = CSSStyleSheet::new(None, document);
//         sheet._location = location.map(|url| url.serialize());
//     }
// }
