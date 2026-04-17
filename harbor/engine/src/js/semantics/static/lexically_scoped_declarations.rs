use crate::js::{
    collect_seq,
    semantics::r#static::{OwnedParseNode, ParseNode, StaticSemantics},
    stmt::{STATEMENT_OR_DECLARATION_DECLARATION, STATEMENT_OR_DECLARATION_STATEMENT},
};

pub fn lexically_scoped_declarations(node: &ParseNode) -> Vec<OwnedParseNode> {
    match node {
        ParseNode::Script(script) => {
            lexically_scoped_declarations(&ParseNode::StatementOrDeclList(&script.body))
        }
        ParseNode::Statement(_) => vec![],
        ParseNode::Declaration(decl) => vec![decl.declaration_part()],
        ParseNode::StatementOrDeclList(seq) => {
            let mut decls = Vec::new();
            let slice = collect_seq(*seq);

            for stmt_or_decl in slice {
                match stmt_or_decl.tag {
                    STATEMENT_OR_DECLARATION_DECLARATION => {
                        let decl = unsafe { *stmt_or_decl.data.declaration };
                        decls.extend(ParseNode::Declaration(&decl).lexically_scoped_declarations());
                    }
                    STATEMENT_OR_DECLARATION_STATEMENT => {}
                    _ => unreachable!(
                        "Unexpected tag in lexically_scoped_declarations: {:?}",
                        stmt_or_decl.tag
                    ),
                }
            }

            decls
        }
        _ => panic!(
            "Unsupported node type for lexically_scoped_declarations: {:?}",
            node
        ),
    }
}

pub fn lexically_scoped_declarations_owned(node: &OwnedParseNode) -> Vec<OwnedParseNode> {
    match node {
        OwnedParseNode::Script(script) => {
            lexically_scoped_declarations_owned(&OwnedParseNode::StatementOrDeclList(script.body))
        }
        OwnedParseNode::Statement(_) => vec![],
        OwnedParseNode::Declaration(decl) => vec![OwnedParseNode::Declaration(decl.clone())],
        OwnedParseNode::StatementOrDeclList(seq) => {
            let mut decls = Vec::new();
            let slice = collect_seq(seq);

            for stmt_or_decl in slice {
                match stmt_or_decl.tag {
                    STATEMENT_OR_DECLARATION_DECLARATION => {
                        let decl = unsafe { *stmt_or_decl.data.declaration };
                        decls.extend(
                            OwnedParseNode::Declaration(decl).lexically_scoped_declarations(),
                        );
                    }
                    STATEMENT_OR_DECLARATION_STATEMENT => {}
                    _ => unreachable!(
                        "Unexpected tag in lexically_scoped_declarations_owned: {:?}",
                        stmt_or_decl.tag
                    ),
                }
            }

            decls
        }
        _ => panic!(
            "Unsupported node type for lexically_scoped_declarations_owned: {:?}",
            node
        ),
    }
}
