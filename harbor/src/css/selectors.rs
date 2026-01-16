use crate::css::tokenize::HashToken;

/// https://www.w3.org/TR/selectors-4/#structure
/// A selector represents a particular pattern of element(s) in a tree structure.
/// The term selector can refer to a simple selector, compound selector, complex selector,
/// or selector list
///
/// NOTE:
/// Selector List is TBI
#[derive(Clone)]
pub enum Selector {
    Simple(String),
    Compound(Vec<Selector>),
    Complex(Vec<Selector>),
}

/// https://www.w3.org/TR/selectors-4/#typedef-ns-prefix
///
/// NOTE:
/// Ends with a `|` in the grammar, but we don't need to store that
pub struct NSPrefix {
    /// NOTE: Should be taken from ident token value or may be `'*'`
    pub prefix: Option<String>,
}

/// https://www.w3.org/TR/selectors-4/#typedef-wq-name
pub struct WQName {
    pub namespace: Option<NSPrefix>,

    /// NOTE: Should be taken from ident token value
    pub local_name: String,
}

pub enum TypeSelector {
    WQName(WQName),

    /// NOTE: Represents `<prefix>|*` in the grammar
    /// `'*'` is implied
    Prefixed(NSPrefix),
}

pub type UniversalSelector = TypeSelector;

pub enum AttributeSelector {
    Exists(WQName),
    /// NOTE: .2 is canonically [ <ident-token> | <string-token> ]
    /// but we just use String here for simplicity
    WithMatcher(WQName, AttrMatcher, String, Option<AttrModifier>),
}

pub enum AttrMatcher {
    Equal,          // =
    Includes,       // ~=
    DashMatch,      // |=
    PrefixMatch,    // ^=
    SuffixMatch,    // $=
    SubstringMatch, // *=
}

pub enum AttrModifier {
    CaseInsensitive, // i
    CaseSensitive,   // s
}

pub type IDSelector = HashToken;

/// NOTE: Should be taken from ident token value
/// Prefixed `.` is implied
pub type ClassSelector = String;

/// NOTE: Prefixed `:` is implied
pub enum PseudoClassSelector {
    Raw(String),

    /// NOTE:
    /// .0 taken from value of CSSToken::Function
    /// .1 is anything
    /// Implied end with `)`
    Function(String, Vec<String>),
}

/// NOTE: Prefixed `::` is implied
type PseudoElementSelector = PseudoClassSelector;

pub enum SimpleSelector {
    TypeSelector(TypeSelector),
    UniversalSelector(UniversalSelector),
    AttributeSelector(AttributeSelector),
    ClassSelector(ClassSelector),
    IDSelector(IDSelector),
    PseudoClassSelector(PseudoClassSelector),
    PseudoElementSelector(PseudoElementSelector),
}
