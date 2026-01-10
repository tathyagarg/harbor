use crate::html5::{
    self,
    dom::*,
    parse::{ElementOrMarker, ParseError, Parser, ParserState},
    tag_groups::*,
};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub is_self_closing: bool,
    pub attributes: Vec<(String, String)>,
}

impl Tag {
    pub fn empty() -> Tag {
        Tag::new(&String::new())
    }

    fn new(name: &String) -> Tag {
        Tag {
            name: name.clone(),
            is_self_closing: false,
            attributes: vec![],
        }
    }

    pub fn attribute_names_iter(&self) -> impl Iterator<Item = String> {
        self.attributes.iter().map(|x| x.0.clone())
    }

    fn attribute_names(&self) -> Vec<String> {
        self.attribute_names_iter().collect()
    }
}

pub enum TagToken {
    Start(Tag),
    End(Tag),
}

impl TagToken {
    fn apply(&mut self, f: impl Fn(&mut Tag) -> Tag) -> Tag {
        match self {
            TagToken::Start(start) => f(start),
            TagToken::End(end) => f(end),
        }
    }

    pub fn apply_no_ret(&mut self, f: impl Fn(&mut Tag)) {
        self.apply(|t: &mut Tag| {
            f(t);
            t.clone()
        });
    }

    pub fn new_tag_attr(&mut self, data: Option<(String, String)>) {
        self.apply_no_ret(|t: &mut Tag| {
            t.attributes
                .push(data.clone().unwrap_or((String::new(), String::new())))
        });
    }

    pub fn push_to_attr_name(&mut self, ch: char) {
        self.apply_no_ret(|t: &mut Tag| t.attributes.last_mut().unwrap().0.push(ch));
    }

    pub fn push_to_attr_val(&mut self, ch: char) {
        self.apply_no_ret(|t: &mut Tag| t.attributes.last_mut().unwrap().1.push(ch));
    }

    pub fn set_self_closing(&mut self) {
        self.apply_no_ret(|t: &mut Tag| t.is_self_closing = true);
    }

    pub fn current_attr(&mut self) -> (String, String) {
        self.apply(|t| t.clone()).attributes.last().unwrap().clone()
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct DOCTYPE {
    pub name: Option<String>,
    pub public_identifier: Option<String>,
    pub system_identifier: Option<String>,

    force_quirks: bool,
}

impl DOCTYPE {
    pub fn set_quirks(&mut self) {
        self.force_quirks = true;
    }

    pub fn with_name(&self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self.clone()
        }
    }

    pub fn with_empty_name(&self) -> Self {
        self.with_name(String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    DOCTYPE(DOCTYPE),
    StartTag(Tag),
    EndTag(Tag),
    Comment(String),
    Character(char),
    EOF,
}

#[derive(Debug, Clone)]
pub enum InsertMode {
    Initial,
    BeforeHTML,
    BeforeHead,
    InHead,
    InHeadNoScript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

impl InsertMode {
    pub fn handle_initial(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}') => {}
            Token::Comment(data) => {
                parser._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(doctype) => {
                if doctype.name.as_ref().unwrap().to_ascii_lowercase() != "html"
                    || doctype.public_identifier.is_some()
                    || doctype
                        .system_identifier
                        .as_ref()
                        .is_some_and(|idtfr| idtfr.as_str() != "about:legacy-compat")
                {
                    parser.error(ParseError::UnexpectedCharacterAfterDOCTYPESystemIdentifier);
                }

                let doctype = DocumentType::new(
                    doctype.name.unwrap_or(String::new()).as_str(),
                    // NOTE: this will be overwritten by append child anyway, so clone is
                    // fine
                    Some(Rc::clone(&parser.document.document)),
                )
                .with_public_id(doctype.public_identifier.unwrap_or(String::new()).as_str())
                .with_system_id(doctype.system_identifier.unwrap_or(String::new()).as_str());

                Node::append_child(
                    &Rc::clone(&parser.document.document_mut()._node),
                    Rc::new(RefCell::new(NodeKind::DocumentType(doctype))),
                );

                // TODO: Set quirks mode if needed:
                // Reference: https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode

                parser.insertion_mode = InsertMode::BeforeHTML;
            }
            _ => {
                // TODO: Set quirks mode:

                parser.insertion_mode = InsertMode::BeforeHTML;
                return false;
            }
        }

        return true;
    }

    fn handle_before_html(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::DOCTYPE(_) => {
                parser.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in before html insertion mode",
                ));
            }
            Token::Comment(data) => {
                parser._insert_comment(data.as_str(), None);
            }
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}') => {}
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                let element = Element::from_token(
                    &token,
                    html5::HTML_NAMESPACE,
                    &NodeKind::Document(parser.document.document().clone()),
                );
                let document = parser.document.document_mut();
                let element_node = Rc::new(RefCell::new(NodeKind::Element(Rc::clone(&element))));

                Node::append_child(&Rc::clone(&document._node), element_node);

                parser.open_elements_stack.push(element);

                parser.insertion_mode = InsertMode::BeforeHead;
            }
            Token::EndTag(Tag { name, .. })
                if !matches!(name.as_str(), "head" | "body" | "html" | "br") =>
            {
                parser.error(ParseError::Custom(
                    "Unexpected end tag token in before html insertion mode",
                ));
            }
            _ => {
                let element = Element::from_token(
                    &Token::StartTag(Tag::new(&String::from("html"))),
                    "html",
                    &NodeKind::Document(parser.document.document().clone()),
                );

                let document = parser.document.document_mut();

                Node::append_child(
                    &Rc::clone(&document._node),
                    Rc::new(RefCell::new(NodeKind::Element(element.clone()))),
                );

                parser.open_elements_stack.push(element);

                parser.insertion_mode = InsertMode::BeforeHead;

                return false;
            }
        }

        return true;
    }

    fn handle_before_head(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}') => {}
            Token::Comment(data) => {
                parser._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                parser.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in before head insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(parser, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "head" => {
                let head = parser.open_elements_stack.insert_html_element(&token);

                parser.head_element_id = Some(head.borrow().id.clone());
                parser.insertion_mode = InsertMode::InHead;
            }
            Token::EndTag(Tag { name, .. })
                if !matches!(name.as_str(), "head" | "body" | "html" | "br") =>
            {
                parser.error(ParseError::Custom(
                    "Unexpected end tag token in before head insertion mode",
                ));
            }
            _ => {
                let head = parser
                    .open_elements_stack
                    .insert_html_element(&Token::StartTag(Tag::new(&String::from("head"))));
                parser.head_element_id = Some(head.borrow().id.clone());
                parser.insertion_mode = InsertMode::InHead;

                return false;
            }
        }

        return true;
    }

    fn handle_in_head(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::Character(ch)
                if matches!(
                    ch,
                    '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
                ) =>
            {
                parser._insert_character(ch);
            }
            Token::Comment(data) => {
                parser._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                parser.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in in head insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(parser, token);
            }
            Token::StartTag(ref tag)
                if matches!(tag.name.as_str(), "base" | "basefont" | "bgsound" | "link") =>
            {
                parser.open_elements_stack.insert_html_element(&token);
                parser.open_elements_stack.pop();
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "meta" => {
                parser.open_elements_stack.insert_html_element(&token);
                parser.open_elements_stack.pop();
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "title" => {
                parser._generic_rcdata_parsing_algorithm(&token);
            }
            Token::StartTag(ref tag)
                if (tag.name.as_str() == "noscript" && parser.flag_scripting)
                    || (matches!(tag.name.as_str(), "noframes" | "style")) =>
            {
                parser._generic_text_parsing_algorithm(&token);
            }
            Token::StartTag(ref tag)
                if (tag.name.as_str() == "noscript" && !parser.flag_scripting) =>
            {
                parser.open_elements_stack.insert_html_element(&token);
                parser.insertion_mode = InsertMode::InHeadNoScript;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "script" => {
                let element = Element::from_token(
                    &token,
                    html5::HTML_NAMESPACE,
                    &NodeKind::Element(
                        parser
                            .open_elements_stack
                            .adjusted_current_node()
                            .unwrap()
                            .clone(),
                    ),
                );

                // TODO: Handle script element parsing correctly

                parser
                    .open_elements_stack
                    .appropriate_insertion_place(None)
                    .insert(&mut NodeKind::Element(element.clone()));

                parser.open_elements_stack.push(element);
                parser.state = ParserState::ScriptData;

                parser.original_insertion_mode = Some(parser.insertion_mode.clone());
                parser.insertion_mode = InsertMode::Text;
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "head" => {
                parser.open_elements_stack.pop();
                parser.insertion_mode = InsertMode::AfterHead;
            }
            // TODO: Start Tag "template", End Tag "template"
            Token::StartTag(ref start) if start.name.as_str() == "head" => {
                parser.error(ParseError::Custom(
                    "Unexpected start tag token in in head insertion mode",
                ));
            }
            Token::EndTag(ref tag) if !matches!(tag.name.as_str(), "body" | "html" | "br") => {
                parser.error(ParseError::Custom(
                    "Unexpected end tag token in in head insertion mode",
                ));
            }
            // }
            _ => {
                parser.open_elements_stack.pop();
                parser.insertion_mode = InsertMode::AfterHead;
                return false;
            }
        }

        return true;
    }

    fn handle_in_head_noscript(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::DOCTYPE(_) => {
                parser.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in in head noscript insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(parser, token);
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "noscript" => {
                parser.open_elements_stack.pop();
                parser.insertion_mode = InsertMode::InHead;
            }
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}')
            | Token::Comment(_) => {
                InsertMode::handle_in_head(parser, token);
            }
            Token::StartTag(ref tag)
                if matches!(tag.name.as_str(), "base" | "basefont" | "bgsound" | "link") =>
            {
                InsertMode::handle_in_head(parser, token);
            }
            Token::StartTag(ref tag) if matches!(tag.name.as_str(), "head" | "noscript") => {
                parser.error(ParseError::Custom(
                    "Unexpected start tag token in in head noscript insertion mode",
                ));
            }
            Token::EndTag(ref tag) if tag.name.as_str() != "br" => {
                parser.error(ParseError::Custom(
                    "Unexpected end tag token in in head noscript insertion mode",
                ));
            }
            _ => {
                parser.error(ParseError::Custom(
                    "Anything else token in in head noscript insertion mode",
                ));

                parser.open_elements_stack.pop();
                parser.insertion_mode = InsertMode::InHead;
                return false;
            }
        }

        return true;
    }

    fn handle_after_head(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::Character(ch)
                if matches!(
                    ch,
                    '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
                ) =>
            {
                parser._insert_character(ch);
            }
            Token::Comment(data) => {
                parser._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                parser.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in after head insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(parser, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "body" => {
                parser.open_elements_stack.insert_html_element(&token);
                parser.flag_frameset_ok = false;

                parser.insertion_mode = InsertMode::InBody;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "frameset" => {
                parser.open_elements_stack.insert_html_element(&token);
                parser.insertion_mode = InsertMode::InFrameset;
            }
            Token::StartTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                parser.error(ParseError::Custom(
                    "Unexpected start tag token in after head insertion mode",
                ));

                // TODO: Process the token using the rules for the "in head" insertion mode
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "template" => {
                InsertMode::handle_in_head(parser, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "head" => {
                parser.error(ParseError::Custom(
                    "Unexpected start tag token in after head insertion mode",
                ));
            }
            Token::EndTag(ref tag) if !matches!(tag.name.as_str(), "body" | "html" | "br") => {
                parser.error(ParseError::Custom(
                    "Unexpected end tag token in after head insertion mode",
                ));
            }
            _ => {
                parser
                    .open_elements_stack
                    .insert_html_element(&Token::StartTag(Tag::new(&String::from("body"))));
                parser.insertion_mode = InsertMode::InBody;

                return false;
            }
        }

        return true;
    }

    fn handle_in_body(parser: &mut Parser, token: Token) -> bool {
        // NOTE: Oh boy - this is a chonker
        match token {
            Token::Character('\u{0000}') => {
                parser.error(ParseError::UnexpectedNullCharacter);
            }
            Token::Character(ch)
                if matches!(
                    ch,
                    '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
                ) =>
            {
                parser._reconstruct_active_formatting_elements();
                parser._insert_character(ch);
            }
            Token::Character(ch) => {
                parser._reconstruct_active_formatting_elements();
                parser._insert_character(ch);
                parser.flag_frameset_ok = false;
            }
            Token::Comment(data) => {
                parser._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                parser.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in in body insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                parser.error(ParseError::Custom(
                    "Unexpected html start tag token in in body insertion mode",
                ));

                // TODO:
                // If there is a template element on the stack of open elements, then ignore the token.
                // Otherwise, for each attribute on the token, check to see if the attribute is already
                // present on the top element of the stack of open elements. If it is not, add the attribute
                // and its corresponding value to that element.
            }
            Token::StartTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                InsertMode::handle_in_head(parser, token);
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "template" => {
                InsertMode::handle_in_head(parser, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "body" => {
                parser.error(ParseError::Custom(
                    "Unexpected body start tag token in in body insertion mode",
                ));

                if parser.open_elements_stack.elements.len() == 1
                    || parser.open_elements_stack.elements[1]
                        .borrow()
                        .qualified_name()
                        != "body"
                    || parser._is_element_on_open_elements("template")
                {
                    // ignore
                    return true;
                } else {
                    parser.flag_frameset_ok = false;

                    let body_element = &parser.open_elements_stack.elements[1];

                    for (attr_name, attr_value) in &tag.attributes {
                        if !body_element
                            .borrow()
                            .attributes()
                            .iter()
                            .any(|attr| attr.local_name() == attr_name.as_str())
                        {
                            Element::push_attr_raw_rc(
                                &body_element,
                                attr_name.as_str(),
                                attr_value.as_str(),
                            );
                            // body_element.push_attr_raw(attr_name.as_str(), attr_value.as_str());
                        }
                    }
                }
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "frameset" => {
                parser.error(ParseError::Custom(
                    "Unexpected frameset start tag token in in body insertion mode",
                ));

                if parser.open_elements_stack.elements.len() == 1
                    || parser.open_elements_stack.elements[1]
                        .borrow()
                        .qualified_name()
                        != "body"
                    || !parser.flag_frameset_ok
                {
                    return true;
                } else {
                    let second = &parser.open_elements_stack.elements[1];

                    let position = second
                        .borrow()
                        .node()
                        .borrow()
                        .parent_node()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .position_of_child(&NodeKind::Element(second.clone()));

                    second
                        .borrow()
                        .node()
                        .borrow_mut()
                        .parent_node_mut()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .pop_child(position);

                    parser.open_elements_stack.elements.drain(1..);
                    parser.open_elements_stack.insert_html_element(&token);

                    parser.insertion_mode = InsertMode::InFrameset;
                }
            }
            Token::EOF => {
                if !parser.open_elements_stack.elements.is_empty() {
                    InsertMode::handle_in_template(parser, token);
                } else {
                    if parser
                        .open_elements_stack
                        .has_non_special_element_in_scope()
                    {
                        parser.error(ParseError::Custom(
                            "Unexpected EOF token in in body insertion mode",
                        ));
                    }

                    return true;
                }
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "body" => {
                if parser
                    .open_elements_stack
                    .has_non_special_element_in_scope()
                {
                    parser.error(ParseError::Custom(
                        "Unexpected end tag token in in body insertion mode",
                    ));
                }

                parser.insertion_mode = InsertMode::AfterBody;
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "html" => {
                if parser
                    .open_elements_stack
                    .has_non_special_element_in_scope()
                {
                    parser.error(ParseError::Custom(
                        "Unexpected end tag token in in body insertion mode",
                    ));
                }

                parser.insertion_mode = InsertMode::AfterBody;

                return false;
            }
            Token::StartTag(ref tag)
                if ARBITRARY_SPECIAL_GROUP_START.contains(&tag.name.as_str()) =>
            {
                if parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.open_elements_stack.close_p_tag();
                }

                parser.open_elements_stack.insert_html_element(&token);
            }
            Token::StartTag(ref tag)
                if matches!(tag.name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") =>
            {
                if parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.open_elements_stack.close_p_tag();
                }

                if parser
                    .open_elements_stack
                    .adjusted_current_node()
                    .is_some_and(|el| {
                        matches!(
                            el.borrow().qualified_name().as_str(),
                            "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                        )
                    })
                {
                    parser.error(ParseError::Custom(
                        "Unexpected heading start tag token in in body insertion mode",
                    ));

                    parser.open_elements_stack.pop();
                }

                parser.open_elements_stack.insert_html_element(&token);
            }
            Token::StartTag(ref tag) if matches!(tag.name.as_str(), "pre" | "listing") => {
                if parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.open_elements_stack.close_p_tag();
                }

                parser.open_elements_stack.insert_html_element(&token);

                // TODO:
                // If the next token is a U+000A LINE FEED (LF) character token,
                // then ignore that token and move on to the next one.
                // (Newlines at the start of `pre` blocks are ignored as an authoring convenience.)

                parser.flag_frameset_ok = false;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "form" => {
                todo!("Handle form start tag in in body insertion mode");
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "li" => {
                parser.flag_frameset_ok = false;

                let mut node_index = parser.open_elements_stack.elements.len();
                let mut node = parser.open_elements_stack.nth(node_index);

                loop {
                    if node
                        .as_ref()
                        .is_some_and(|el| el.borrow().qualified_name() == "li")
                    {
                        parser
                            .open_elements_stack
                            .generate_implied_end_tags(Some("li"));

                        if parser
                            .open_elements_stack
                            .adjusted_current_node()
                            .is_some_and(|el| el.borrow().qualified_name() != "li")
                        {
                            parser.error(ParseError::Custom(
                                "Unexpected current node after generating implied end tags for li",
                            ));
                        }

                        parser.open_elements_stack.pop_until("li");
                        break;
                    }

                    if node.as_ref().is_some_and(|el| {
                        el.borrow().is_special_excluding(&["address", "div", "p"])
                    }) {
                        break;
                    } else {
                        node_index -= 1;
                        node = parser.open_elements_stack.nth(node_index);
                    }
                }

                if parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.open_elements_stack.close_p_tag();
                }

                parser.open_elements_stack.insert_html_element(&token);
            }
            Token::StartTag(ref tag) if matches!(tag.name.as_str(), "dd" | "dt") => {
                parser.flag_frameset_ok = false;

                let mut node_index = parser.open_elements_stack.elements.len();
                let mut node = parser.open_elements_stack.nth(node_index);

                loop {
                    if node
                        .as_ref()
                        .is_some_and(|el| el.borrow().qualified_name() == "dd")
                    {
                        parser
                            .open_elements_stack
                            .generate_implied_end_tags(Some("dd"));

                        if parser
                            .open_elements_stack
                            .adjusted_current_node()
                            .is_some_and(|el| el.borrow().qualified_name() != "dd")
                        {
                            parser.error(ParseError::Custom(
                                "Unexpected current node after generating implied end tags for dd",
                            ));
                        }

                        parser.open_elements_stack.pop_until("dd");
                        break;
                    }

                    if node
                        .as_ref()
                        .is_some_and(|el| el.borrow().qualified_name() == "dt")
                    {
                        parser
                            .open_elements_stack
                            .generate_implied_end_tags(Some("dt"));

                        if parser
                            .open_elements_stack
                            .adjusted_current_node()
                            .is_some_and(|el| el.borrow().qualified_name() != "dt")
                        {
                            parser.error(ParseError::Custom(
                                "Unexpected current node after generating implied end tags for dt",
                            ));
                        }

                        parser.open_elements_stack.pop_until("dt");
                        break;
                    }

                    if node.as_ref().is_some_and(|el| {
                        el.borrow().is_special_excluding(&["address", "div", "p"])
                    }) {
                        break;
                    } else {
                        node_index -= 1;
                        node = parser.open_elements_stack.nth(node_index);
                    }
                }

                if parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.open_elements_stack.close_p_tag();
                }

                parser.open_elements_stack.insert_html_element(&token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "plaintext" => {
                if parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.open_elements_stack.close_p_tag();
                }

                parser.open_elements_stack.insert_html_element(&token);

                parser.state = ParserState::PLAINTEXT;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "button" => {
                if parser
                    .open_elements_stack
                    .has_element_in_default_scope("button")
                {
                    parser.error(ParseError::Custom(
                        "Unexpected button start tag token in in body insertion mode",
                    ));

                    parser.open_elements_stack.generate_implied_end_tags(None);
                    parser.open_elements_stack.pop_until("button");
                }

                parser._reconstruct_active_formatting_elements();
                parser.open_elements_stack.insert_html_element(&token);
                parser.flag_frameset_ok = false;
            }
            Token::EndTag(ref tag) if ARBITRARY_SPECIAL_GROUP_END.contains(&tag.name.as_str()) => {
                if !parser
                    .open_elements_stack
                    .has_element_in_default_scope(&tag.name)
                {
                    parser.error(ParseError::Custom(
                        "Unexpected special end tag token in in body insertion mode",
                    ));

                    return true;
                }

                parser.open_elements_stack.generate_implied_end_tags(None);

                if parser
                    .open_elements_stack
                    .adjusted_current_node()
                    .is_some_and(|el| el.borrow().qualified_name() != tag.name)
                {
                    // println!(
                    //     "Current node: {:#?}",
                    //     parser.open_elements_stack.adjusted_current_node()
                    // );
                    // println!("Expected node: {}", tag.name);
                    parser.error(ParseError::Custom(
                        "Unexpected current node after generating implied end tags for special end tag",
                    ));
                }

                parser.open_elements_stack.pop_until(&tag.name);
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "form" => {
                todo!("Handle form end tag in in body insertion mode");
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "p" => {
                if !parser.open_elements_stack.has_element_in_button_scope("p") {
                    parser.error(ParseError::Custom(
                        "Unexpected p end tag token in in body insertion mode",
                    ));

                    parser
                        .open_elements_stack
                        .insert_html_element(&Token::StartTag(Tag::new(&String::from("p"))));
                }

                parser.open_elements_stack.close_p_tag();
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "li" => {
                if !parser
                    .open_elements_stack
                    .has_element_in_list_item_scope("li")
                {
                    parser.error(ParseError::Custom(
                        "Unexpected li end tag token in in body insertion mode",
                    ));

                    return true;
                }

                parser
                    .open_elements_stack
                    .generate_implied_end_tags(Some("li"));

                if parser
                    .open_elements_stack
                    .adjusted_current_node()
                    .is_some_and(|el| el.borrow().qualified_name() != "li")
                {
                    println!(
                        "Current node: {:#?}",
                        parser.open_elements_stack.adjusted_current_node()
                    );

                    parser.error(ParseError::Custom(
                        "Unexpected current node after generating implied end tags for li end tag",
                    ));
                }

                parser.open_elements_stack.pop_until("li");
            }
            Token::EndTag(ref tag) if matches!(tag.name.as_str(), "dd" | "dt") => {
                if !parser
                    .open_elements_stack
                    .has_element_in_default_scope(&tag.name)
                {
                    parser.error(ParseError::Custom(
                        "Unexpected dd/dt end tag token in in body insertion mode",
                    ));

                    return true;
                }

                parser
                    .open_elements_stack
                    .generate_implied_end_tags(Some(&tag.name));

                if parser
                    .open_elements_stack
                    .adjusted_current_node()
                    .is_some_and(|el| el.borrow().qualified_name() != tag.name)
                {
                    parser.error(ParseError::Custom(
                        "Unexpected current node after generating implied end tags for dd/dt end tag",
                    ));
                }

                parser.open_elements_stack.pop_until(&tag.name);
            }
            Token::EndTag(ref tag)
                if matches!(tag.name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") =>
            {
                if !parser
                    .open_elements_stack
                    .has_element_in_default_scope(&tag.name)
                {
                    parser.error(ParseError::Custom(
                        "Unexpected heading end tag token in in body insertion mode",
                    ));

                    return true;
                }

                parser.open_elements_stack.generate_implied_end_tags(None);

                if parser
                    .open_elements_stack
                    .adjusted_current_node()
                    .is_some_and(|el| el.borrow().qualified_name() != tag.name)
                {
                    parser.error(ParseError::Custom(
                        "Unexpected current node after generating implied end tags for heading end tag",
                    ));
                }

                parser.open_elements_stack.pop_until(&tag.name);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "a" => {
                let last_marker_pos = parser.active_formatting_elements.last_marker();
                let relevant_slice = parser.active_formatting_elements.elements
                    [last_marker_pos.unwrap_or(0)..]
                    .to_vec();

                if relevant_slice.iter().any(|el| match el {
                    ElementOrMarker::Element(e) => e.borrow().qualified_name() == "a",
                    ElementOrMarker::Marker => false,
                }) {
                    parser.error(ParseError::Custom(
                        "Unexpected a start tag token in in body insertion mode",
                    ));

                    InsertMode::_adoption_agency(parser, tag);
                }

                parser._reconstruct_active_formatting_elements();
                let element = parser.open_elements_stack.insert_html_element(&token);
                parser.active_formatting_elements.push(element);
            }
            Token::StartTag(ref tag) if FORMATTING_ELEMENT_NAMES.contains(&tag.name.as_str()) => {
                parser._reconstruct_active_formatting_elements();
                let element = parser.open_elements_stack.insert_html_element(&token);
                parser.active_formatting_elements.push(element);
            }
            Token::EndTag(ref tag)
                if FORMATTING_ELEMENT_NAMES.contains(&tag.name.as_str())
                    || tag.name.as_str() == "a" =>
            {
                InsertMode::_adoption_agency(parser, tag);
            }
            _ => {
                todo!("Handle other tokens in in body insertion mode: {:?}", token);
            }
        }

        return true;
    }

    fn handle_text(parser: &mut Parser, token: Token) -> bool {
        match token {
            Token::Character(ch) => {
                parser._insert_character(ch);
            }
            Token::EOF => {
                parser.error(ParseError::Custom(
                    "Unexpected EOF token in text insertion mode",
                ));

                parser.open_elements_stack.pop();
                parser.insertion_mode = parser.original_insertion_mode.clone().unwrap();
                return false;
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "script" => {
                todo!("Handle script end tag correctly");
            }
            Token::EndTag(_) => {
                parser.open_elements_stack.pop();
                parser.insertion_mode = parser.original_insertion_mode.clone().unwrap();
            }
            _ => {
                unreachable!("Unexpected token in text insertion mode");
            }
        }

        return true;
    }

    fn handle_in_template(parser: &mut Parser, token: Token) -> bool {
        todo!("Implement in template insertion mode");

        match token {
            _ => {}
        }

        return true;
    }

    /// Let subject be token's tag name.
    /// If the current node is an HTML element whose tag name is subject, and the current node is
    /// not in the list of active formatting elements, then pop the current node off the stack of
    /// open elements and return.
    /// Let outerLoopCounter be 0.
    /// While true:
    ///     If outerLoopCounter is greater than or equal to 8, then return.
    ///     Increment outerLoopCounter by 1.
    ///     Let formattingElement be the last element in the list of active formatting elements that:
    ///         is between the end of the list and the last marker in the list, if any, or the start of the list otherwise, and
    ///         has the tag name subject.
    ///     If there is no such element, then return and instead act as described in the "any other end tag" entry above.
    ///     If formattingElement is not in the stack of open elements, then this is a parse error; remove the element from the list, and return.
    ///     If formattingElement is in the stack of open elements, but the element is not in scope, then this is a parse error; return.
    ///     If formattingElement is not the current node, this is a parse error. (But do not return.)
    ///     Let furthestBlock be the topmost node in the stack of open elements that is lower in the stack than formattingElement, and is an element in the special category. There might not be one.
    ///     If there is no furthestBlock, then the UA must first pop all the nodes from the bottom of the stack of open elements, from the current node up to and including formattingElement, then remove formattingElement from the list of active formatting elements, and finally return.
    ///     Let commonAncestor be the element immediately above formattingElement in the stack of open elements.
    ///     Let a bookmark note the position of formattingElement in the list of active formatting elements relative to the elements on either side of it in the list.
    ///     Let node and lastNode be furthestBlock.
    ///     Let innerLoopCounter be 0.
    ///     While true:
    ///         Increment innerLoopCounter by 1.
    ///         Let node be the element immediately above node in the stack of open elements, or if node is no longer in the stack of open elements (e.g. because it got removed by this algorithm), the element that was immediately above node in the stack of open elements before node was removed.
    ///         Append lastNode to node.
    ///         Set lastNode to node.
    ///     Insert whatever lastNode ended up being in the previous step at the appropriate place for inserting a node, but using commonAncestor as the override target.
    ///     Create an element for the token for which formattingElement was created, in the HTML namespace, with furthestBlock as the intended parent.
    ///     Take all of the child nodes of furthestBlock and append them to the element created in the last step.
    ///     Append that new element to furthestBlock.
    ///     Remove formattingElement from the list of active formatting elements, and insert the new element into the list of active formatting elements at the position of the aforementioned bookmark.
    ///     Remove formattingElement from the stack of open elements, and insert the new element into the stack of open elements immediately below the position of furthestBlock in that stack.
    fn _adoption_agency(parser: &mut Parser, tag: &Tag) {
        let subject = tag.name.as_str();

        if parser.open_elements_stack.current_node().is_some_and(|el| {
            el.borrow().qualified_name() == subject
                && !parser.active_formatting_elements.contains(&el)
        }) {
            parser.open_elements_stack.pop();
            return;
        }

        let mut outer_loop_counter = 0;

        loop {
            if outer_loop_counter >= 8 {
                return;
            }

            outer_loop_counter += 1;

            let last_marker_pos = parser.active_formatting_elements.last_marker();
            let relevant_slice =
                parser.active_formatting_elements.elements[last_marker_pos.unwrap_or(0)..].to_vec();

            let formatting_element_pos = relevant_slice.iter().rposition(|el| match el {
                ElementOrMarker::Element(e) => e.borrow().qualified_name() == subject,
                ElementOrMarker::Marker => false,
            });
            let adjusted_formatting_element_pos =
                formatting_element_pos.map(|pos| pos + last_marker_pos.unwrap_or(0));

            let _formatting_element = formatting_element_pos.map(|pos| relevant_slice[pos].clone());

            if _formatting_element.is_none() {
                // TODO: Come up with a way to singal "any other end tag" handling
                return;
            }

            let formatting_element = match _formatting_element.unwrap() {
                ElementOrMarker::Element(e) => e,
                ElementOrMarker::Marker => {
                    panic!("Formatting element cannot be a marker");
                }
            };

            if !parser.open_elements_stack.contains_rc(&formatting_element) {
                parser.error(ParseError::Custom(
                    "Formatting element not in open elements stack during adoption agency algorithm",
                ));

                // Remove the element from the list
                parser
                    .active_formatting_elements
                    .elements
                    .retain(|el| match el {
                        ElementOrMarker::Element(e) => !Rc::ptr_eq(&e, &formatting_element),
                        ElementOrMarker::Marker => true,
                    });

                return;
            }

            if !parser
                .open_elements_stack
                .has_element_in_default_scope(&subject)
            {
                parser.error(ParseError::Custom(
                    "Formatting element not in scope during adoption agency algorithm",
                ));

                return;
            }

            if !parser
                .open_elements_stack
                .current_node()
                .is_some_and(|el| Rc::ptr_eq(&el, &formatting_element))
            {
                parser.error(ParseError::Custom(
                    "Formatting element not current node during adoption agency algorithm",
                ));
            }

            let furthest_block_pos = relevant_slice[formatting_element_pos.unwrap() + 1..]
                .iter()
                .position(|el| match el {
                    ElementOrMarker::Element(e) => e.borrow().is_special(),
                    ElementOrMarker::Marker => false,
                });
            let adjusted_furthest_block_pos =
                furthest_block_pos.map(|pos| pos + adjusted_formatting_element_pos.unwrap() + 1);

            let _furthest_block = furthest_block_pos.map(|el_idx| {
                match &relevant_slice[el_idx + formatting_element_pos.unwrap() + 1] {
                    ElementOrMarker::Element(e) => Rc::clone(&e),
                    ElementOrMarker::Marker => {
                        panic!("Furthest block cannot be a marker");
                    }
                }
            });

            if _furthest_block.is_none() {
                parser.open_elements_stack.pop_until(&subject);

                parser
                    .active_formatting_elements
                    .elements
                    .retain(|el| match el {
                        ElementOrMarker::Element(e) => !Rc::ptr_eq(&e, &formatting_element),
                        ElementOrMarker::Marker => true,
                    });

                return;
            }

            let furthest_block = _furthest_block.unwrap();

            let common_ancestor = parser
                .open_elements_stack
                .nth(adjusted_formatting_element_pos.unwrap() - 1)
                .unwrap();

            let mut bookmark = adjusted_formatting_element_pos.unwrap();

            let mut node_index = adjusted_furthest_block_pos.unwrap();
            let last_node_index = adjusted_furthest_block_pos.unwrap();

            let mut node = Rc::clone(parser.open_elements_stack.nth(node_index).as_ref().unwrap());
            let mut last_node = Rc::clone(
                parser
                    .open_elements_stack
                    .nth(last_node_index)
                    .as_ref()
                    .unwrap(),
            );

            let mut inner_loop_counter = 0;

            loop {
                inner_loop_counter += 1;

                node_index -= 1;
                node = Rc::clone(parser.open_elements_stack.nth(node_index).as_ref().unwrap());

                if Rc::ptr_eq(&node, &formatting_element) {
                    break;
                }

                if inner_loop_counter > 3 && !parser.open_elements_stack.contains_rc(&node) {
                    parser
                        .active_formatting_elements
                        .elements
                        .retain(|el| match el {
                            ElementOrMarker::Element(e) => !Rc::ptr_eq(&e, &node),
                            ElementOrMarker::Marker => true,
                        });
                }

                let common_ancestor_node_kind = NodeKind::Element(Rc::clone(&common_ancestor));

                let element = Element::from_token(
                    node.borrow().token().unwrap(),
                    html5::HTML_NAMESPACE,
                    &common_ancestor_node_kind,
                );

                parser.active_formatting_elements.elements[bookmark] =
                    ElementOrMarker::Element(Rc::clone(&element));
                parser.open_elements_stack.elements[adjusted_formatting_element_pos.unwrap() - 1] =
                    Rc::clone(&element);

                node = Rc::clone(&element);

                if Rc::ptr_eq(&last_node, &furthest_block) {
                    bookmark = node_index + 1;
                }

                Node::append_child(
                    &node.borrow().node(),
                    Rc::new(RefCell::new(NodeKind::Element(Rc::clone(&last_node)))),
                );

                last_node = Rc::clone(&node);
            }

            parser
                .open_elements_stack
                .appropriate_insertion_place(Some(Rc::clone(&common_ancestor)))
                .insert(&mut NodeKind::Element(last_node.clone()));

            let new_element = Element::from_token(
                formatting_element.borrow().token().unwrap(),
                html5::HTML_NAMESPACE,
                &NodeKind::Element(Rc::clone(&furthest_block)),
            );

            furthest_block
                .borrow()
                .node()
                .borrow()
                .child_nodes()
                .map(|child| {
                    Node::append_child(&new_element.borrow().node(), Rc::clone(&child));
                });

            Node::append_child(
                &furthest_block.borrow().node(),
                Rc::new(RefCell::new(NodeKind::Element(Rc::clone(&new_element)))),
            );

            parser
                .active_formatting_elements
                .elements
                .retain(|el| match el {
                    ElementOrMarker::Element(e) => !Rc::ptr_eq(&e, &formatting_element),
                    ElementOrMarker::Marker => true,
                });

            parser
                .active_formatting_elements
                .elements
                .insert(bookmark, ElementOrMarker::Element(Rc::clone(&new_element)));

            parser
                .open_elements_stack
                .elements
                .retain(|el| !Rc::ptr_eq(el, &formatting_element));

            parser.open_elements_stack.elements.insert(
                adjusted_furthest_block_pos.unwrap() + 1,
                Rc::clone(&new_element),
            );
        }
    }

    pub fn handle(&self, parser: &mut Parser, token: &Token) -> bool {
        let token = token.clone();

        match self {
            InsertMode::Initial => InsertMode::handle_initial(parser, token),
            InsertMode::BeforeHTML => InsertMode::handle_before_html(parser, token),
            InsertMode::BeforeHead => InsertMode::handle_before_head(parser, token),
            InsertMode::InHead => InsertMode::handle_in_head(parser, token),
            InsertMode::InHeadNoScript => InsertMode::handle_in_head_noscript(parser, token),
            InsertMode::AfterHead => InsertMode::handle_after_head(parser, token),
            InsertMode::InBody => InsertMode::handle_in_body(parser, token),
            InsertMode::Text => InsertMode::handle_text(parser, token),
            _ => {
                true
                // todo!("Handle insertion mode {:?}", self);
            }
        }
    }
}
