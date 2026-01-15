use std::{cell::RefCell, rc::Weak};

use crate::{
    css::{
        parser::{AtRule, ComponentValue, preprocess},
        tokenize::{CSSToken, tokenize_from_string},
    },
    html5::dom::{Document, Element},
    http::url::{Serializable, URL},
    infra::InputStream,
};

pub enum DeclarationOrAtRule {
    Declaration(CSSDeclaration),
    AtRule(AtRule),
}

pub struct CSSDeclaration {
    /// The property name of the declaration.
    pub property_name: String,

    /// The value of the declaration represented as a list of component values.
    pub value: Vec<ComponentValue>,

    /// Either set or unset. Can be changed.
    pub important: bool,

    /// Set if the property name is defined to be case-sensitive according to its
    /// specification, otherwise unset.
    pub case_sensitive: bool,
}

impl CSSDeclaration {
    pub fn new(name: String, value: Vec<ComponentValue>) -> Self {
        CSSDeclaration {
            property_name: name,
            value,
            important: false,
            case_sensitive: false,
        }
    }
}

pub struct CSSDeclarationBlock {
    /// Set if the object is a computed style declaration, rather than a specified style.
    /// Unless otherwise stated it is unset.
    pub _computed: bool,

    /// The CSS declarations associated with the object.
    pub _declarations: Vec<CSSDeclaration>,

    /// The CSS rule that the CSS declaration block is associated with, if any, or null otherwise.
    pub _parent_rule: Option<Box<CSSRule>>,

    /// The Element that the CSS declaration block is associated with, if any, or null otherwise.
    pub _owner_element: Option<Weak<RefCell<Element>>>,

    /// Unset by default. Set when the CSS declaration block is updating the owner node’s style
    /// attribute.
    pub _updating: bool,
}

impl CSSDeclarationBlock {
    pub fn new(owner_element: Option<Weak<RefCell<Element>>>) -> Self {
        let mut block = CSSDeclarationBlock {
            _computed: false,
            _declarations: Vec::new(),
            _parent_rule: None,
            _owner_element: owner_element,
            _updating: false,
        };

        if block._owner_element.is_none() || block._computed {
            return block;
        }

        let value = block
            ._owner_element
            .as_ref()
            .and_then(|weak_elem| weak_elem.upgrade())
            .map(|elem| {
                elem.borrow()
                    .get_attribute("style")
                    .map(|attr_val| attr_val.to_string())
                    .unwrap_or_default()
            });

        todo!()
    }
}

pub enum CSSRuleType {
    Unknown = 0,
    Style = 1,
    Charset = 2,
    Import = 3,
    Media = 4,
    FontFace = 5,
    Page = 6,
    Keyframes = 7,
    Keyframe = 8,
    Margin = 9,
    Namespace = 10,
    CounterStyle = 11,
    Supports = 12,
    FontFeatureValues = 14,
    Viewport = 15,
}

pub struct CSSRule {
    /// A non-negative integer associated with a particular type of rule.
    /// This item is initialized when a rule is created and cannot change.
    pub _type: CSSRuleType,

    /// A text representation of the rule suitable for direct use in a style sheet.
    /// This item is initialized when a rule is created and can be changed.
    _text: String,

    /// A reference to an enclosing CSS rule or null. If the rule has an enclosing rule when
    /// it is created, then this item is initialized to the enclosing rule; otherwise it is null.
    /// It can be changed to null.
    _parent_rule: Option<Box<CSSRule>>,

    /// A reference to a parent CSS style sheet or null. This item is initialized to reference
    /// an associated style sheet when the rule is created. It can be changed to null.
    _parent_style_sheet: Option<Weak<RefCell<CSSStyleSheet>>>,

    /// A list of child CSS rules. The list can be mutated.
    pub _css_rules: Vec<CSSRule>,
}

pub trait CSSRuleExt {
    fn text(&self) -> &String;

    fn parent_rule(&self) -> &Option<Box<CSSRule>>;
    fn parent_style_sheet(&self) -> &Option<Weak<RefCell<CSSStyleSheet>>>;

    fn _type(&self) -> &CSSRuleType;
}

impl CSSRuleExt for CSSRule {
    fn text(&self) -> &String {
        &self._text
    }

    fn parent_rule(&self) -> &Option<Box<CSSRule>> {
        &self._parent_rule
    }

    fn parent_style_sheet(&self) -> &Option<Weak<RefCell<CSSStyleSheet>>> {
        &self._parent_style_sheet
    }

    fn _type(&self) -> &CSSRuleType {
        &self._type
    }
}

pub trait CSSStyleRule {
    fn selector_text(&self) -> &String;

    // TODO: Change to proper CSSStyleDeclaration
    fn style(&self) -> &Vec<(String, String)>;
}

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

pub trait StyleSheet {
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
pub struct CSSStyleSheetInit {
    base_url: Option<String>,
    media: Option<MediaList>,
    disabled: Option<bool>,
}

pub trait CSSStyleSheetExt {
    fn new(options: Option<CSSStyleSheetInit>, document: Weak<RefCell<Document>>) -> Self;

    fn owner_rule(&self) -> &Option<CSSRule>;

    fn css_rules(&self) -> &Vec<CSSRule>;
    fn css_rules_mut(&mut self) -> &mut Vec<CSSRule>;

    fn insert_rule(&mut self, rule: Vec<CSSToken>, index: Option<usize>) -> Result<usize, String>;
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

    fn css_rules_mut(&mut self) -> &mut Vec<CSSRule> {
        &mut self._css_rules
    }

    fn insert_rule(&mut self, rule: Vec<CSSToken>, index: Option<usize>) -> Result<usize, String> {
        // Placeholder implementation
        let idx = index.unwrap_or(self._css_rules.len());
        if idx > self._css_rules.len() {
            return Err("Index out of bounds".to_string());
        }
        todo!("Parse CSSToken vector into CSSRule and insert into stylesheet");
        // self._css_rules.insert(idx, CSSRule);
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

impl CSSStyleSheet {
    pub fn set_location_url(&mut self, location: URL) {
        self._location = Some(location.serialize());
    }

    pub fn set_location(&mut self, location: String) {
        self._location = Some(location);
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
