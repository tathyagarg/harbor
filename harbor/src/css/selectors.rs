#![allow(dead_code)]

use crate::{
    css::tokenize::{CSSToken, HashToken},
    infra::InputStream,
};

/// https://www.w3.org/TR/selectors-4/#structure
/// A selector represents a particular pattern of element(s) in a tree structure.
/// The term selector can refer to a simple selector, compound selector, complex selector,
/// or selector list
///
/// NOTE:
/// Selector List is TBI
#[derive(Debug, Clone)]
pub enum Selector {
    Simple(String),
    Compound(Vec<Selector>),
    Complex(Vec<Selector>),
}

/// https://www.w3.org/TR/selectors-4/#typedef-ns-prefix
///
/// NOTE:
/// Ends with a `|` in the grammar, but we don't need to store that
#[derive(Debug, Clone)]
pub struct NSPrefix {
    /// NOTE: Should be taken from ident token value or may be `'*'`
    pub prefix: Option<String>,
}

/// https://www.w3.org/TR/selectors-4/#typedef-wq-name
#[derive(Debug, Clone)]
pub struct WQName {
    pub namespace: Option<NSPrefix>,

    /// NOTE: Should be taken from ident token value
    pub local_name: String,
}

#[derive(Debug, Clone)]
pub enum TypeSelector {
    WQName(WQName),

    /// NOTE: Represents `<prefix>|*` in the grammar
    /// `'*'` is implied
    Prefixed(NSPrefix),
}

pub type UniversalSelector = TypeSelector;

#[derive(Debug, Clone)]
pub enum AttributeSelector {
    Exists(WQName),
    /// NOTE: .2 is canonically [ <ident-token> | <string-token> ]
    /// but we just use String here for simplicity
    WithMatcher(WQName, AttrMatcher, String, Option<AttrModifier>),
}

#[derive(Debug, Clone)]
pub enum AttrMatcher {
    Equal,          // =
    Includes,       // ~=
    DashMatch,      // |=
    PrefixMatch,    // ^=
    SuffixMatch,    // $=
    SubstringMatch, // *=
}

#[derive(Debug, Clone)]
pub enum AttrModifier {
    CaseInsensitive, // i
    CaseSensitive,   // s
}

pub type IDSelector = HashToken;

/// NOTE: Should be taken from ident token value
/// Prefixed `.` is implied
pub type ClassSelector = String;

/// NOTE: Prefixed `:` is implied
#[derive(Debug, Clone)]
pub enum PseudoClassSelector {
    Raw(String),

    /// NOTE:
    /// .0 taken from value of CSSToken::Function
    /// .1 is anything
    /// Implied end with `)`
    Function(String, Vec<CSSToken>),
}

/// NOTE: Prefixed `::` is implied
type PseudoElementSelector = PseudoClassSelector;

#[derive(Debug, Clone)]
pub enum SubclassSelector {
    IDSelector(IDSelector),
    ClassSelector(ClassSelector),
    AttributeSelector(AttributeSelector),
    PseudoClassSelector(PseudoClassSelector),
    PseudoElementSelector(PseudoElementSelector),
}

pub enum SimpleSelector {
    TypeSelector(TypeSelector),
    UniversalSelector(UniversalSelector),
    SubclassSelector(SubclassSelector),
}

#[derive(Debug, Clone)]
pub struct CompoundSelector {
    pub type_selector: Option<TypeSelector>,
    pub subclass_selectors: Vec<SubclassSelector>,
    pub pseudo_selectors: Vec<(PseudoElementSelector, Vec<PseudoClassSelector>)>,
}

#[derive(Debug, Clone)]
pub enum Combinator {
    Child,        // >
    NextSibling,  // +
    LaterSibling, // ~
    /// NOTE: Technically not part of the specified combinators
    Descendant, // (whitespace)
                  // Tables Combinator is TBI
}

#[derive(Debug, Clone)]
pub struct ComplexSelector {
    pub compound: CompoundSelector,
    pub combinators: Vec<(Combinator, CompoundSelector)>,
}

pub type ComplexSelectorList = Vec<ComplexSelector>;

pub type SelectorList = ComplexSelectorList;

/// <combinator> = '>' | '+' | '~' | [ '|' '|' ]
fn parse_combinator(tokens: &mut InputStream<CSSToken>) -> Option<Combinator> {
    match tokens.peek() {
        Some(CSSToken::Delim('>')) => {
            tokens.consume();
            Some(Combinator::Child)
        }
        Some(CSSToken::Delim('+')) => {
            tokens.consume();
            Some(Combinator::NextSibling)
        }
        Some(CSSToken::Delim('~')) => {
            tokens.consume();
            Some(Combinator::LaterSibling)
        }
        _ => None,
    }
}

fn parse_ns_prefix(tokens: &mut InputStream<CSSToken>) -> Option<NSPrefix> {
    let curr_tokens = tokens.clone();

    fn matched(tokens: &mut InputStream<CSSToken>, prefix: String) -> Option<NSPrefix> {
        tokens.consume(); // Consume the '|'

        let ns_prefix = NSPrefix {
            prefix: Some(prefix),
        };
        Some(ns_prefix)
    }

    if let Some(CSSToken::Ident(ident)) = tokens.peek() {
        let prefix = ident.clone();
        tokens.consume();

        if let Some(CSSToken::Delim('|')) = tokens.peek() {
            return matched(tokens, prefix);
        } else {
            *tokens = curr_tokens;
            return None;
        }
    } else if let Some(CSSToken::Delim('*')) = tokens.peek() {
        tokens.consume();

        if let Some(CSSToken::Delim('|')) = tokens.peek() {
            return matched(tokens, "*".to_string());
        } else {
            *tokens = curr_tokens;
            return None;
        }
    }

    None
}

fn parse_wq_name(tokens: &mut InputStream<CSSToken>) -> Option<WQName> {
    let curr_tokens = tokens.clone();

    let namespace = parse_ns_prefix(tokens);

    if let Some(CSSToken::Ident(ident)) = tokens.peek() {
        tokens.consume();

        let local_name = ident.clone();

        let wq_name = WQName {
            namespace,
            local_name,
        };
        return Some(wq_name);
    }

    *tokens = curr_tokens;
    None
}

/// <type-selector> = <wq-name> | <ns-prefix>? '*'
fn parse_type_selector(tokens: &mut InputStream<CSSToken>) -> Option<TypeSelector> {
    let curr_tokens = tokens.clone();

    if let Some(wq_name) = parse_wq_name(tokens) {
        return Some(TypeSelector::WQName(wq_name));
    }

    if let Some(ns_prefix) = parse_ns_prefix(tokens) {
        if let Some(CSSToken::Delim('*')) = tokens.peek() {
            tokens.consume();

            return Some(TypeSelector::Prefixed(ns_prefix));
        }
    }

    *tokens = curr_tokens;
    None
}

fn parse_attr_matcher(tokens: &mut InputStream<CSSToken>) -> Option<AttrMatcher> {
    let curr_tokens = tokens.clone();

    fn match_eq(tokens: &mut InputStream<CSSToken>) -> bool {
        if let Some(CSSToken::Delim('=')) = tokens.peek() {
            tokens.consume();
            return true;
        }

        false
    }

    match tokens.peek() {
        Some(CSSToken::Delim('~')) => {
            tokens.consume();
            if match_eq(tokens) {
                return Some(AttrMatcher::Includes);
            }
        }
        Some(CSSToken::Delim('|')) => {
            tokens.consume();
            if match_eq(tokens) {
                return Some(AttrMatcher::DashMatch);
            }
        }
        Some(CSSToken::Delim('^')) => {
            tokens.consume();
            if match_eq(tokens) {
                return Some(AttrMatcher::PrefixMatch);
            }
        }
        Some(CSSToken::Delim('$')) => {
            tokens.consume();
            if match_eq(tokens) {
                return Some(AttrMatcher::SuffixMatch);
            }
        }
        Some(CSSToken::Delim('*')) => {
            tokens.consume();
            if match_eq(tokens) {
                return Some(AttrMatcher::SubstringMatch);
            }
        }
        Some(CSSToken::Delim('=')) => {
            tokens.consume();
            return Some(AttrMatcher::Equal);
        }
        _ => {
            return None;
        }
    }

    *tokens = curr_tokens;
    None
}

fn parse_attribute_modifier(tokens: &mut InputStream<CSSToken>) -> Option<AttrModifier> {
    if let Some(CSSToken::Ident(ident)) = tokens.peek() {
        match ident.as_str() {
            "i" => {
                tokens.consume();
                return Some(AttrModifier::CaseInsensitive);
            }
            "s" => {
                tokens.consume();
                return Some(AttrModifier::CaseSensitive);
            }
            _ => {}
        }
    }

    None
}

fn parse_pseudo_class_selector(tokens: &mut InputStream<CSSToken>) -> Option<PseudoClassSelector> {
    let curr_tokens = tokens.clone();

    if let Some(CSSToken::Colon) = tokens.peek() {
        tokens.consume();

        if let Some(CSSToken::Ident(ident)) = tokens.peek() {
            tokens.consume();

            let name = ident.clone();
            return Some(PseudoClassSelector::Raw(name));
        } else if let Some(CSSToken::Function(func_name)) = tokens.peek() {
            tokens.consume();

            let name = func_name.clone();
            let mut args = Vec::new();

            while let Some(token) = tokens.peek() {
                match token {
                    CSSToken::RightParenthesis => {
                        tokens.consume(); // Consume the ')'
                        break;
                    }
                    _ if !matches!(token, CSSToken::BadString | CSSToken::BadURL) => {
                        args.push(token.clone());
                        tokens.consume();
                    }
                    _ => {
                        *tokens = curr_tokens;
                        return None;
                    }
                }
            }

            return Some(PseudoClassSelector::Function(name, args));
        }
    }

    *tokens = curr_tokens;
    None
}

/// Holy mother of god
/// <attribute-selector> = '[' <wq-name> ']' |
///                    '[' <wq-name> <attr-matcher> [ <string-token> | <ident-token> ] <attr-modifier>? ']'
fn parse_attribute_selector(tokens: &mut InputStream<CSSToken>) -> Option<AttributeSelector> {
    // Save current state, so we can fuck around with tokens as much as we want
    let curr_tokens = tokens.clone();

    if let Some(CSSToken::LeftSquareBracket) = tokens.peek() {
        tokens.consume();

        if let Some(wq_name) = parse_wq_name(tokens) {
            if let Some(CSSToken::RightSquareBracket) = tokens.peek() {
                tokens.consume();
                return Some(AttributeSelector::Exists(wq_name));
            }

            if let Some(attr_matcher) = parse_attr_matcher(tokens) {
                if let Some(CSSToken::Ident(val) | CSSToken::String(val)) = tokens.peek() {
                    tokens.consume();

                    let modifier = parse_attribute_modifier(tokens);

                    if let Some(CSSToken::RightSquareBracket) = tokens.peek() {
                        tokens.consume();

                        return Some(AttributeSelector::WithMatcher(
                            wq_name,
                            attr_matcher,
                            val,
                            modifier,
                        ));
                    }
                }
            }
        }
    }

    *tokens = curr_tokens;
    None
}

/// <id-selector> = <hash-token>
fn parse_id_selector(tokens: &mut InputStream<CSSToken>) -> Option<IDSelector> {
    if let Some(CSSToken::Hash(hash_token)) = tokens.peek() {
        tokens.consume();

        let id_token = hash_token.clone();
        return Some(id_token);
    }

    None
}

/// <class-selector> = '.' <ident-token>
fn parse_class_selector(tokens: &mut InputStream<CSSToken>) -> Option<ClassSelector> {
    if let Some(CSSToken::Delim('.')) = tokens.peek()
        && let Some(CSSToken::Ident(ident)) = tokens.peek_nth(1)
    {
        tokens.consume();
        tokens.consume();

        let class_name = ident.clone();
        return Some(class_name);
    }

    None
}

/// <subclass-selector> = <id-selector> | <class-selector> |
///                   <attribute-selector> | <pseudo-class-selector>
fn parse_subclass_selector(tokens: &mut InputStream<CSSToken>) -> Option<SubclassSelector> {
    if let Some(id_selector) = parse_id_selector(tokens) {
        return Some(SubclassSelector::IDSelector(id_selector));
    }

    if let Some(class_selector) = parse_class_selector(tokens) {
        return Some(SubclassSelector::ClassSelector(class_selector));
    }

    if let Some(attribute_selector) = parse_attribute_selector(tokens) {
        return Some(SubclassSelector::AttributeSelector(attribute_selector));
    }

    if let Some(pseudo_class_selector) = parse_pseudo_class_selector(tokens) {
        return Some(SubclassSelector::PseudoClassSelector(pseudo_class_selector));
    }

    None
}

fn parse_pseudo_element_selector(
    tokens: &mut InputStream<CSSToken>,
) -> Option<PseudoElementSelector> {
    let curr_tokens = tokens.clone();

    if let Some(CSSToken::Colon) = tokens.peek() {
        tokens.consume();

        if let Some(pseudo_class) = parse_pseudo_class_selector(tokens) {
            return Some(pseudo_class);
        }
    }

    *tokens = curr_tokens;
    None
}

/// <compound-selector> = [ <type-selector>? <subclass-selector>*
///                     [ <pseudo-element-selector> <pseudo-class-selector>* ]* ]!
fn parse_compound_selector(tokens: &mut InputStream<CSSToken>) -> Option<CompoundSelector> {
    let curr_tokens = tokens.clone();

    let type_selector = parse_type_selector(tokens);

    let mut subclass_selectors = Vec::new();

    while let Some(subclass_selector) = parse_subclass_selector(tokens) {
        subclass_selectors.push(subclass_selector);
    }

    let mut pseudo_selectors = Vec::new();

    loop {
        if let Some(pseudo_element_selector) = parse_pseudo_element_selector(tokens) {
            let mut pseudo_classes = Vec::new();

            while let Some(pseudo_class_selector) = parse_pseudo_class_selector(tokens) {
                pseudo_classes.push(pseudo_class_selector);
            }

            pseudo_selectors.push((pseudo_element_selector, pseudo_classes));
        } else {
            break;
        }
    }

    if !(type_selector.is_some() || !subclass_selectors.is_empty() || !pseudo_selectors.is_empty())
    {
        *tokens = curr_tokens;
        return None;
    }

    Some(CompoundSelector {
        type_selector,
        subclass_selectors,
        pseudo_selectors,
    })
}

/// <complex-selector> = <compound-selector> [ <combinator>? <compound-selector> ]*
fn parse_complex_selector(tokens: &mut InputStream<CSSToken>) -> Option<ComplexSelector> {
    let first_compound = parse_compound_selector(tokens);
    if first_compound.is_none() {
        return None;
    }

    let mut combinators = Vec::new();
    let mut last_save = tokens.clone();

    loop {
        if let Some(combinator) = parse_combinator(tokens) {
            let next_compound = parse_compound_selector(tokens);
            if next_compound.is_none() {
                *tokens = last_save;

                return None;
            }

            combinators.push((combinator, next_compound.unwrap()));
            last_save = tokens.clone();
        } else if let Some(next_compound) = parse_compound_selector(tokens) {
            combinators.push((Combinator::Descendant, next_compound));
            last_save = tokens.clone();
        } else {
            break;
        }
    }

    Some(ComplexSelector {
        compound: first_compound.unwrap(),
        combinators,
    })
}

/// <complex-selector-list> = <complex-selector>#
fn parse_complex_selector_list(tokens: &mut InputStream<CSSToken>) -> Option<ComplexSelectorList> {
    let mut selectors = Vec::new();

    let mut last_save = tokens.clone();

    loop {
        let selector = parse_complex_selector(tokens);
        if selector.is_none() {
            *tokens = last_save;
            break;
        }

        selectors.push(selector.unwrap());

        last_save = tokens.clone();

        if let Some(CSSToken::Comma) = tokens.peek() {
            tokens.consume(); // Consume the comma
        } else {
            break; // No more selectors
        }
    }

    Some(selectors)
}

/// <selector-list> = <complex-selector-list>
pub fn parse_tokens_as_selector_list(tokens: Vec<CSSToken>) -> Option<SelectorList> {
    // remove whitespaces
    let filtered_tokens = tokens
        .into_iter()
        .filter(|t| !matches!(t, CSSToken::Whitespace))
        .collect::<Vec<_>>();

    let mut tokens_stream = InputStream::new(&filtered_tokens[..]);

    parse_complex_selector_list(&mut tokens_stream)
}
