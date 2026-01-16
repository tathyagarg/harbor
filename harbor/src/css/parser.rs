use std::{cell::RefCell, rc::Weak};

use crate::{
    css::{
        cssom::{
            CSSDeclaration, CSSRuleExt, CSSRuleNode, CSSRuleType, CSSStyleRuleData, CSSStyleSheet,
            CSSStyleSheetExt, DeclarationOrAtRule,
        },
        selectors::parse_tokens_as_selector_list,
        tokenize::{CSSToken, tokenize_from_string},
    },
    html5::dom::Document,
    infra::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Function(pub String, pub Vec<ComponentValue>);

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleBlock(pub CSSToken, pub Vec<ComponentValue>);

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    Token(CSSToken),
    Function(Function),
    SimpleBlock(SimpleBlock),
}

#[derive(Debug)]
struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: SimpleBlock,
}

#[derive(Debug)]
pub struct AtRule {
    name: String,
    prelude: Vec<ComponentValue>,
    block: Option<SimpleBlock>,
}

#[derive(Debug)]
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
        if is_surrogate(ch as u32) {
            result.push('\u{FFFD}');
        } else {
            result.push(ch);
        }
    }

    result
}

fn parse_selector_list(tokens: Vec<CSSToken>) {}

fn consume_function(stream: &mut InputStream<CSSToken>) -> Function {
    let name_token = stream.current();

    let name = match name_token {
        CSSToken::Function(func_name) => func_name,
        _ => panic!("Expected function token"),
    };

    let mut function = Function(name, Vec::new());

    loop {
        match stream.consume() {
            None => return function,
            Some(CSSToken::RightParenthesis) => return function,
            Some(_) => {
                stream.reconsume();
                let component_value = consume_component_value(stream);
                function.1.push(component_value);
            }
        }
    }
}

fn normalize_string_to_tokens(input: String) -> Vec<CSSToken> {
    let filtered = preprocess(&input);
    tokenize_from_string(filtered)
}

fn consume_component_value(stream: &mut InputStream<CSSToken>) -> ComponentValue {
    match stream.consume() {
        Some(
            CSSToken::LeftCurlyBracket | CSSToken::LeftSquareBracket | CSSToken::LeftParenthesis,
        ) => {
            let simple_block = consume_simple_block(stream);
            ComponentValue::SimpleBlock(simple_block)
        }
        Some(CSSToken::Function(_)) => {
            let function = consume_function(stream);
            ComponentValue::Function(function)
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

fn consume_declaration_from_cvs(cvs: &mut InputStream<ComponentValue>) -> Option<CSSDeclaration> {
    let consumed = cvs.consume();

    if let Some(ComponentValue::Token(CSSToken::Ident(name))) = consumed {
        let mut declaration = CSSDeclaration::new(name, vec![]);

        while let Some(ComponentValue::Token(CSSToken::Whitespace)) = cvs.peek() {
            cvs.consume();
        }

        if let Some(ComponentValue::Token(CSSToken::Colon)) = cvs.consume() {
            while let Some(ComponentValue::Token(CSSToken::Whitespace)) = cvs.peek() {
                cvs.consume();
            }

            while let Some(component_value) = cvs.peek()
                && !matches!(component_value, ComponentValue::Token(CSSToken::EOF))
            {
                declaration.value.push(cvs.consume().unwrap());
            }

            let last2 = declaration
                .value
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, cv)| !matches!(cv, ComponentValue::Token(CSSToken::Whitespace)))
                .take(2)
                .collect::<Vec<_>>();

            if last2.len() < 2 {
                return Some(declaration);
            }

            let (i1, last) = last2[0].clone();
            let (i2, second_last) = last2[1].clone();

            if last.clone() == ComponentValue::Token(CSSToken::Ident("important".to_string()))
                && second_last.clone() == ComponentValue::Token(CSSToken::Delim('!'))
            {
                // NOTE: Must remove in reverse order to keep indices valid
                declaration.value.remove(i1);
                declaration.value.remove(i2);

                declaration.important = true;
            }

            while let Some(ComponentValue::Token(CSSToken::Whitespace)) = declaration.value.last() {
                declaration.value.pop();
            }

            return Some(declaration);
        } else {
            return None;
        }
    }

    None
}

fn consume_list_of_declarations(stream: &mut InputStream<CSSToken>) -> Vec<DeclarationOrAtRule> {
    let mut declarations = Vec::new();

    loop {
        match stream.consume() {
            Some(CSSToken::Whitespace | CSSToken::Semicolon) => {}
            Some(CSSToken::EOF) | None => return declarations,
            Some(CSSToken::AtKeyword(_)) => {
                stream.reconsume();
                declarations.push(DeclarationOrAtRule::AtRule(consume_at_rule(stream)));
            }
            Some(CSSToken::Ident(_)) => {
                let mut temporary = Vec::new();
                temporary.push(ComponentValue::Token(stream.current()));

                while stream
                    .peek()
                    .is_some_and(|t| t != CSSToken::Semicolon && t != CSSToken::EOF)
                {
                    let component_value = consume_component_value(stream);
                    temporary.push(component_value);
                }

                let mut cvs_stream = InputStream::new(&temporary);

                if let Some(declaration) = consume_declaration_from_cvs(&mut cvs_stream) {
                    declarations.push(DeclarationOrAtRule::Declaration(declaration));
                    temporary.clear();
                }
            }
            _ => {
                stream.reconsume();

                while stream
                    .peek()
                    .is_some_and(|t| t != CSSToken::Semicolon && t != CSSToken::EOF)
                {
                    consume_component_value(stream);
                }
            }
        }
    }
}

fn parse_list_of_declarations(inp: String) -> Vec<DeclarationOrAtRule> {
    let result = normalize_string_to_tokens(inp);
    let mut stream = InputStream::new(&result);
    consume_list_of_declarations(&mut stream)
}

/// TODO: The specification:
/// https://www.w3.org/TR/cssom-1/#parse-a-css-declaration-block
/// In section 3.1, requires:
/// > Let parsed declaration be the result of parsing declaration according to the appropriate CSS
/// specifications, ...
/// However, this is a LONG, and extremely tedious process which will require individual grammars
/// for each CSS property. For now, we will just parse the declaration block into individual declarations
/// without validating them.
pub fn parse_css_declaration_block(input: String) -> Vec<CSSDeclaration> {
    let declarations_or_at_rules = parse_list_of_declarations(input);
    let mut declarations = Vec::new();

    for item in declarations_or_at_rules {
        if let DeclarationOrAtRule::Declaration(decl) = item {
            declarations.push(decl);
        }
    }

    declarations
}

pub fn parse_stylesheet(
    stream: &mut InputStream<CSSToken>,
    document: Weak<RefCell<Document>>,
    location: Option<String>,
) -> CSSStyleSheet {
    let mut stylesheet = CSSStyleSheet::new(None, document);
    stylesheet.set_location(location.unwrap_or_default());

    let mut rules = Vec::<Box<dyn CSSRuleExt>>::new();

    for rule in consume_list_of_rules(stream, true) {
        match rule {
            Rule::AtRule(at_rule) => {
                println!("At-Rule: {:#?}", at_rule);
            }
            Rule::QualifiedRule(qualified_rule) => {
                let prelude = qualified_rule.prelude;

                let prelude_to_tokens = prelude
                    .into_iter()
                    .filter_map(|cv| {
                        if let ComponentValue::Token(token) = cv {
                            Some(token)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<CSSToken>>();

                let selectors =
                    parse_tokens_as_selector_list(prelude_to_tokens).unwrap_or(Vec::new());

                let tokens = &qualified_rule
                    .block
                    .1
                    .into_iter()
                    .filter_map(|cv| {
                        if let ComponentValue::Token(token) = cv {
                            Some(vec![token])
                        } else if let ComponentValue::Function(func) = cv {
                            let mut tokens = vec![CSSToken::Function(func.0)];
                            for arg in func.1 {
                                if let ComponentValue::Token(t) = arg {
                                    tokens.push(t);
                                }
                            }
                            tokens.push(CSSToken::RightParenthesis);
                            Some(tokens)
                        } else if let ComponentValue::SimpleBlock(block) = cv {
                            let mut tokens = vec![block.0.clone()];
                            for item in block.1 {
                                if let ComponentValue::Token(t) = item {
                                    tokens.push(t);
                                }
                            }
                            let ending_token = match block.0 {
                                CSSToken::LeftCurlyBracket => CSSToken::RightCurlyBracket,
                                CSSToken::LeftSquareBracket => CSSToken::RightSquareBracket,
                                CSSToken::LeftParenthesis => CSSToken::RightParenthesis,
                                _ => panic!("Invalid starting token for simple block"),
                            };
                            tokens.push(ending_token);
                            Some(tokens)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect::<Vec<CSSToken>>();

                let declarations = consume_list_of_declarations(&mut InputStream::new(&tokens))
                    .iter()
                    .filter_map(|item| {
                        if let DeclarationOrAtRule::Declaration(decl) = item {
                            Some(decl.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<CSSDeclaration>>();

                let style_rule = CSSRuleNode::<CSSStyleRuleData>::new(
                    CSSRuleType::Style,
                    String::new(),
                    None,
                    None,
                    CSSStyleRuleData::new(selectors, declarations),
                );

                rules.push(Box::new(style_rule) as Box<dyn CSSRuleExt>);
            }
        }
    }

    *stylesheet.css_rules_mut() = rules;

    stylesheet
}
