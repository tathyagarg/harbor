use crate::js::{
    collect_seq,
    semantics::r#static::{OwnedParseNode, ParseNode, StaticSemantics},
    stmt::{
        DECLARATION_LEXICAL_DECLARATION, STATEMENT_OR_DECLARATION_DECLARATION,
        STATEMENT_OR_DECLARATION_STATEMENT, Script, SeqStatementOrDeclaration,
    },
    values::string::JsString,
};

fn lexically_declared_names_script(script: &Script) -> Vec<JsString> {
    top_level_lexically_declared_names(&script.body)
}

fn top_level_lexically_declared_names(statements: &SeqStatementOrDeclaration) -> Vec<JsString> {
    let mut names = Vec::new();
    let slice = collect_seq(statements);

    for stmt_or_decl in slice {
        match stmt_or_decl.tag {
            STATEMENT_OR_DECLARATION_DECLARATION => {
                let decl = unsafe { *stmt_or_decl.data.declaration };

                match decl.tag {
                    DECLARATION_LEXICAL_DECLARATION => {
                        let lexical_decl = unsafe { *decl.data.lex_decl };
                        names.extend(ParseNode::LexicalDeclaration(&lexical_decl).bound_names());
                    }
                    _ => {}
                }
            }
            STATEMENT_OR_DECLARATION_STATEMENT => {}
            _ => unreachable!(
                "Unexpected tag in top_level_lexically_declared_names: {:?}",
                stmt_or_decl.tag
            ),
        }
    }

    names
}

pub fn lexically_declared_names(node: &ParseNode) -> Vec<JsString> {
    match node {
        ParseNode::Script(script) => lexically_declared_names_script(script),
        ParseNode::Statement(_) => vec![],
        ParseNode::Declaration(decl) => ParseNode::Declaration(decl).bound_names(),
        _ => panic!(
            "Unsupported node type for lexically_declared_names: {:?}",
            node
        ),
    }
}

pub fn lexically_declared_names_owned(node: &OwnedParseNode) -> Vec<JsString> {
    match node {
        OwnedParseNode::Script(script) => lexically_declared_names_script(script),
        OwnedParseNode::Statement(_) => vec![],
        OwnedParseNode::Declaration(decl) => ParseNode::Declaration(decl).bound_names(),
        _ => panic!(
            "Unsupported node type for lexically_declared_names_owned: {:?}",
            node
        ),
    }
}
