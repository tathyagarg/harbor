use std::{cell::RefCell, rc::Weak};

use crate::{
    css::{
        models::{CSSStyleSheet, CSSStyleSheetExt},
        tokenize::CSSToken,
    },
    html5::dom::Document,
    infra::{self, *},
};

struct Function(String, Vec<ComponentValue>);
struct SimpleBlock(CSSToken, Vec<ComponentValue>);

pub enum ComponentValue {
    Token(CSSToken),
    Function(Function),
    SimpleBlock(SimpleBlock),
}

struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: SimpleBlock,
}

struct AtRule {
    name: String,
    prelude: Vec<ComponentValue>,
    block: Option<SimpleBlock>,
}

enum Rule {
    QualifiedRule(QualifiedRule),
    AtRule(AtRule),
}

pub fn preprocess(input: &String) -> String {
    let step_1 = input
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .replace("\u{000C}", "\n")
        .replace("\u{0000}", "\u{FFFD}");

    let mut result = String::new();

    for ch in step_1.chars() {
        if infra::is_surrogate(ch as u32) {
            result.push('\u{FFFD}');
        } else {
            result.push(ch);
        }
    }

    result
}

fn consume_component_value(stream: &mut InputStream<CSSToken>) -> ComponentValue {
    match stream.consume() {
        Some(token @ CSSToken::LeftCurlyBracket)
        | Some(token @ CSSToken::LeftSquareBracket)
        | Some(token @ CSSToken::LeftParenthesis) => {
            let simple_block = consume_simple_block(stream);
            ComponentValue::SimpleBlock(simple_block)
        }
        Some(CSSToken::Function(_)) => {
            stream.reconsume();
            todo!("Consume a function");
        }
        Some(token) => ComponentValue::Token(token),
        None => panic!("No more tokens to consume"),
    }
}

fn consume_simple_block(stream: &mut InputStream<CSSToken>) -> SimpleBlock {
    let starting_token = stream.current();
    let ending_token = match starting_token {
        CSSToken::LeftCurlyBracket => CSSToken::RightCurlyBracket,
        CSSToken::LeftSquareBracket => CSSToken::RightSquareBracket,
        CSSToken::LeftParenthesis => CSSToken::RightParenthesis,
        _ => panic!("Invalid starting token for simple block"),
    };

    let mut simple_block = SimpleBlock(starting_token.clone(), Vec::new());

    loop {
        match stream.consume() {
            None => return simple_block,
            Some(token) if token == ending_token => return simple_block,
            Some(_) => {
                stream.reconsume();
                let component_value = consume_component_value(stream);

                simple_block.1.push(component_value);
            }
        }
    }
}

fn consume_a_qualified_rule(stream: &mut InputStream<CSSToken>) -> Option<QualifiedRule> {
    let mut qualified_rule = QualifiedRule {
        prelude: Vec::new(),
        block: SimpleBlock(CSSToken::LeftCurlyBracket, Vec::new()),
    };

    loop {
        match stream.consume() {
            None => return None,
            Some(CSSToken::LeftCurlyBracket) => {
                qualified_rule.block = consume_simple_block(stream);
                return Some(qualified_rule);
            }
            _ => {
                stream.reconsume();
                let component_value = consume_component_value(stream);
                qualified_rule.prelude.push(component_value);
            }
        }
    }
}

fn consume_at_rule(stream: &mut InputStream<CSSToken>) -> AtRule {
    let mut at_rule = AtRule {
        name: String::new(),
        prelude: Vec::new(),
        block: None,
    };

    at_rule.name = match stream.consume() {
        Some(name) if !name.string_value().is_empty() => name.string_value(),
        _ => panic!("At-rule must have a name"),
    };

    loop {
        match stream.consume() {
            None => return at_rule,
            Some(CSSToken::Semicolon) => return at_rule,
            Some(CSSToken::LeftCurlyBracket) => {
                at_rule.block = Some(consume_simple_block(stream));
                return at_rule;
            }
            _ => {
                stream.reconsume();
                let component_value = consume_component_value(stream);
                at_rule.prelude.push(component_value);
            }
        }
    }
}

fn consume_list_of_rules(stream: &mut InputStream<CSSToken>, top_level: bool) -> Vec<Rule> {
    let mut rules = Vec::new();

    loop {
        match stream.consume() {
            Some(CSSToken::Whitespace) => continue,
            None | Some(CSSToken::EOF) => return rules,
            Some(CSSToken::CDO | CSSToken::CDC) => {
                if top_level {
                    continue;
                } else {
                    stream.reconsume();
                    if let Some(qualified_rule) = consume_a_qualified_rule(stream) {
                        rules.push(Rule::QualifiedRule(qualified_rule));
                    }
                }
            }
            Some(CSSToken::AtKeyword(_)) => {
                stream.reconsume();
                let at_rule = consume_at_rule(stream);
                rules.push(Rule::AtRule(at_rule));
            }
            _ => {
                stream.reconsume();
                if let Some(qualified_rule) = consume_a_qualified_rule(stream) {
                    rules.push(Rule::QualifiedRule(qualified_rule));
                }
            }
        }
    }
}

pub fn parse(
    stream: &mut InputStream<CSSToken>,
    document: Weak<RefCell<Document>>,
    location: Option<String>,
) {
    let mut stylesheet = CSSStyleSheet::new(None, document);
    stylesheet.set_location(location.unwrap_or_default());

    // *stylesheet.css_rules_mut() = consume_list_of_rules(streamtrue);
}
