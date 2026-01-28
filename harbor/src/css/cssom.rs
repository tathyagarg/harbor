#![allow(unused_variables)]

use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use crate::{
    css::{
        colors::{Color, is_color},
        parser::{AtRule, ComponentValue, parse_css_declaration_block},
        properties::{Background, Display, Font, Margin, Position, WidthValue},
        selectors::SelectorList,
        tokenize::{CSSToken, Dimension},
        values::angles::{is_angle_unit, to_canonical_angle},
    },
    html5::dom::{Document, Element},
    http::url::URL,
    infra::Serializable,
};

impl Serializable for ComponentValue {
    fn serialize(&self) -> String {
        match self {
            ComponentValue::Token(CSSToken::Ident(keyword)) => keyword.to_ascii_lowercase(),
            ComponentValue::Token(CSSToken::Dimension(Dimension { value, unit, .. }))
                if is_angle_unit(unit.as_str()) =>
            {
                let deg_value = to_canonical_angle(*value, unit);
                format!("{}deg", deg_value.unwrap_or(*value))
            }
            comp if is_color(comp) => comp.serialize(),
            // ComponentValue::Function(Function(name, args)) if is_color_function(name)
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum DeclarationOrAtRule {
    Declaration(CSSDeclaration),
    AtRule(AtRule),
}

#[derive(Debug, Clone)]
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

impl Serializable for CSSDeclaration {
    fn serialize(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.property_name);
        s.push_str(": ");
        for cv in &self.value {
            s.push_str(&cv.serialize());
        }

        if self.important {
            s.push_str(" !important");
        }

        s.push(';');

        s
    }
}

/// https://www.w3.org/TR/cssom-1/#css-declaration-block
pub struct CSSDeclarationBlock {
    /// Set if the object is a computed style declaration, rather than a specified style.
    /// Unless otherwise stated it is unset.
    pub _computed: bool,

    /// The CSS declarations associated with the object.
    pub _declarations: Vec<CSSDeclaration>,

    /// The CSS rule that the CSS declaration block is associated with, if any, or null otherwise.
    pub _parent_rule: Option<Box<dyn CSSRuleExt>>,

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

        if value.is_some() {
            let declarations = parse_css_declaration_block(value.unwrap());
            block._declarations = declarations;
        }

        block
    }
}

/// https://www.w3.org/TR/cssom-1/#the-cssstyledeclaration-interface
pub trait CSSStyleDeclaration {
    fn css_text(&self) -> String;
    fn set_css_text(&mut self, text: String) -> Result<(), String>;
}

impl CSSStyleDeclaration for CSSDeclarationBlock {
    fn css_text(&self) -> String {
        if self._computed {
            return String::new();
        }

        todo!()
    }

    fn set_css_text(&mut self, text: String) -> Result<(), String> {
        todo!("Parse CSS text and update declarations");
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CSSRuleNode<T>
where
    T: Clone,
{
    /// A non-negative integer associated with a particular type of rule.
    /// This item is initialized when a rule is created and cannot change.
    pub _type: CSSRuleType,

    /// A text representation of the rule suitable for direct use in a style sheet.
    /// This item is initialized when a rule is created and can be changed.
    _text: String,

    /// A reference to an enclosing CSS rule or null. If the rule has an enclosing rule when
    /// it is created, then this item is initialized to the enclosing rule; otherwise it is null.
    /// It can be changed to null.
    _parent_rule: Option<Box<dyn CSSRuleExt>>,

    /// A reference to a parent CSS style sheet or null. This item is initialized to reference
    /// an associated style sheet when the rule is created. It can be changed to null.
    _parent_style_sheet: Option<Weak<RefCell<CSSStyleSheet>>>,

    /// A list of child CSS rules. The list can be mutated.
    pub _css_rules: Vec<Box<dyn CSSRuleExt>>,

    pub payload: T,
}

pub trait CSSRuleExt: CSSRuleExtClone + Debug {
    fn text(&self) -> &String;

    fn parent_rule(&self) -> &Option<Box<dyn CSSRuleExt>>;
    fn parent_style_sheet(&self) -> &Option<Weak<RefCell<CSSStyleSheet>>>;

    fn _type(&self) -> &CSSRuleType;

    fn as_any(&self) -> &dyn Any;
}

pub trait CSSRuleExtClone {
    fn clone_box(&self) -> Box<dyn CSSRuleExt>;
}

impl<T> CSSRuleExtClone for CSSRuleNode<T>
where
    T: Debug + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn CSSRuleExt> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CSSRuleExt> {
    fn clone(&self) -> Box<dyn CSSRuleExt> {
        self.clone_box()
    }
}

impl<T> CSSRuleExt for CSSRuleNode<T>
where
    T: Debug + Clone + 'static,
{
    fn text(&self) -> &String {
        &self._text
    }

    fn parent_rule(&self) -> &Option<Box<dyn CSSRuleExt>> {
        &self._parent_rule
    }

    fn parent_style_sheet(&self) -> &Option<Weak<RefCell<CSSStyleSheet>>> {
        &self._parent_style_sheet
    }

    fn _type(&self) -> &CSSRuleType {
        &self._type
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T> CSSRuleNode<T>
where
    T: Clone,
{
    pub fn new(
        rule_type: CSSRuleType,
        text: String,
        parent_rule: Option<Box<dyn CSSRuleExt>>,
        parent_style_sheet: Option<Weak<RefCell<CSSStyleSheet>>>,
        payload: T,
    ) -> Self {
        CSSRuleNode {
            _type: rule_type,
            _text: text,
            _parent_rule: parent_rule,
            _parent_style_sheet: parent_style_sheet,
            _css_rules: Vec::new(),
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CSSStyleRuleData {
    pub selectors: SelectorList,

    declarations: Vec<CSSDeclaration>,
}

impl CSSStyleRuleData {
    pub fn new(selectors: SelectorList, declarations: Vec<CSSDeclaration>) -> Self {
        CSSStyleRuleData {
            selectors,
            declarations,
        }
    }
}

impl CSSRuleNode<CSSStyleRuleData> {
    pub fn selector_text(&self) -> String {
        todo!("Serialize selectors to string");
    }

    pub fn set_selector_text(&mut self, text: String) -> Result<(), String> {
        todo!("Parse selector text and update selectors");
    }

    pub fn style(&self) -> Box<dyn CSSStyleDeclaration> {
        Box::new(CSSDeclarationBlock {
            _computed: false,
            _declarations: self.payload.declarations.clone(),
            _parent_rule: Some(Box::new(self.clone())),
            _owner_element: None,
            _updating: false,
        })
    }

    pub fn selectors(&self) -> &SelectorList {
        &self.payload.selectors
    }

    pub fn declarations(&self) -> &Vec<CSSDeclaration> {
        &self.payload.declarations
    }
}

#[derive(Clone)]
pub struct CSSImportRuleData {
    _href: String,

    _style_sheet: Box<CSSStyleSheet>,
}

impl CSSRuleNode<CSSImportRuleData> {
    pub fn href(&self) -> &String {
        &self.payload._href
    }

    pub fn style_sheet(&self) -> &Box<CSSStyleSheet> {
        &self.payload._style_sheet
    }

    pub fn media(&self) -> &MediaList {
        &self.payload._style_sheet._media
    }
}

#[derive(Clone)]
pub struct CSSGroupingRuleData {}

pub trait CSSGroupingRuleExt {
    fn css_rules(&self) -> &Vec<Box<dyn CSSRuleExt>>;

    fn insert_rule(
        &mut self,
        rule: Vec<Box<dyn CSSRuleExt>>,
        index: Option<usize>,
    ) -> Result<usize, String>;

    fn delete_rule(&mut self, index: usize) -> Result<(), String>;
}

impl CSSGroupingRuleExt for CSSRuleNode<CSSGroupingRuleData> {
    fn css_rules(&self) -> &Vec<Box<dyn CSSRuleExt>> {
        &self._css_rules
    }

    fn insert_rule(
        &mut self,
        rule: Vec<Box<dyn CSSRuleExt>>,
        index: Option<usize>,
    ) -> Result<usize, String> {
        eprintln!(
            "Inserting rule into grouping rule: Ensure compliance with spec: https://www.w3.org/TR/cssom-1/#insert-a-css-rule"
        );
        let idx = index.unwrap_or(self._css_rules.len());
        if idx > self._css_rules.len() {
            return Err("Index out of bounds".to_string());
        }

        for r in rule {
            self._css_rules.insert(idx, r);
        }

        Ok(idx)
    }

    fn delete_rule(&mut self, index: usize) -> Result<(), String> {
        if index >= self._css_rules.len() {
            return Err("Index out of bounds".to_string());
        }
        self._css_rules.remove(index);
        Ok(())
    }
}

// TODO: Implement CSSMediaRule
// https://www.w3.org/TR/css-conditional-3/#the-cssmediarule-interface

// TODO: Implement CSSPageRule
// https://www.w3.org/TR/cssom-1/#the-csspagerule-interface
// https://www.w3.org/TR/css-page-3/#page-selectors

// TODO: Implement CSSMarginRule
// https://www.w3.org/TR/cssom-1/#the-cssmarginrule-interface

#[derive(Clone)]
pub struct CSSNamespaceRuleData {
    _prefix: Option<String>,

    _namespace_uri: String,
}

impl CSSRuleNode<CSSNamespaceRuleData> {
    pub fn prefix(&self) -> String {
        self.payload
            ._prefix
            .as_ref()
            .unwrap_or(&String::new())
            .clone()
    }

    pub fn namespace_uri(&self) -> &String {
        &self.payload._namespace_uri
    }
}

#[derive(Debug, Clone)]
pub struct MediaList;

#[derive(Debug, Clone)]
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
    _owner_rule: Option<Box<dyn CSSRuleExt>>,

    /// Specified when created. The MediaList object associated with the CSS style sheet.
    _media: MediaList,

    /// Specified when created. The title of the CSS style sheet, which can be the empty string.
    _title: String,

    /// Specified when created. Either set or unset. Unset by default.
    _alternate: bool,

    /// Either set or unset. Unset by default.
    _disabled: bool,

    /// The CSS rules associated with the CSS style sheet.
    _css_rules: Vec<Box<dyn CSSRuleExt>>,

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
        self._media = media;
        todo!("Update underlying CSS rules to reflect media change");
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

    fn owner_rule(&self) -> &Option<Box<dyn CSSRuleExt>>;

    fn css_rules(&self) -> &Vec<Box<dyn CSSRuleExt>>;
    fn css_rules_mut(&mut self) -> &mut Vec<Box<dyn CSSRuleExt>>;

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

    fn owner_rule(&self) -> &Option<Box<dyn CSSRuleExt>> {
        &self._owner_rule
    }

    fn css_rules(&self) -> &Vec<Box<dyn CSSRuleExt>> {
        &self._css_rules
    }

    fn css_rules_mut(&mut self) -> &mut Vec<Box<dyn CSSRuleExt>> {
        &mut self._css_rules
    }

    fn insert_rule(&mut self, rule: Vec<CSSToken>, index: Option<usize>) -> Result<usize, String> {
        // Placeholder implementation
        let idx = index.unwrap_or(self._css_rules.len());
        if idx > self._css_rules.len() {
            return Err("Index out of bounds".to_string());
        }

        // self._css_rules.insert(idx, CSSRule);
        // Ok(idx)

        todo!("Parse CSSToken vector into CSSRule and insert into stylesheet");
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

#[derive(Debug, Clone)]
pub struct StyleSheetList {
    pub style_sheets: Vec<Rc<RefCell<CSSStyleSheet>>>,
}

impl StyleSheetList {
    pub fn item(&self, index: usize) -> Option<Rc<RefCell<CSSStyleSheet>>> {
        self.style_sheets.get(index).cloned()
    }

    pub fn length(&self) -> usize {
        self.style_sheets.len()
    }
}

#[derive(Debug, Clone)]
pub struct DocumentOrShadowRootStyle {
    pub style_sheets: StyleSheetList,
    // TODO: Implement adopted style sheets
    // pub adopted_style_sheets: Vec<Rc<RefCell<CSSStyleSheet>>>,
}

impl PartialEq for DocumentOrShadowRootStyle {
    fn eq(&self, other: &Self) -> bool {
        false
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for DocumentOrShadowRootStyle {}

#[derive(Default, Clone, Debug)]
pub struct ComputedStyle {
    pub color: Color,
    pub background: Background,
    pub font: Font,

    pub display: Display,
    pub position: Position,

    pub margin: Margin,

    pub width: WidthValue,
}

impl ComputedStyle {
    pub fn inherit(&self) -> Self {
        Self {
            color: self.color.clone(),
            font: self.font.clone(),
            ..Default::default()
        }
    }
}

impl PartialEq for ComputedStyle {
    fn eq(&self, other: &Self) -> bool {
        true
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for ComputedStyle {}
