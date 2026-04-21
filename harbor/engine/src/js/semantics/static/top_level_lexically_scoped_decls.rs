use crate::js::{
    collect_seq,
    semantics::r#static::{OwnedParseNode, ParseNode},
    stmt::{DECLARATION_LEXICAL_DECLARATION, STATEMENT_OR_DECLARATION_DECLARATION},
};

pub fn top_level_lexically_scoped_decls(node: &ParseNode) -> Vec<OwnedParseNode> {
    match node {
        ParseNode::StatementOrDeclList(seq) => {
            let mut decls = Vec::new();
            let slice = collect_seq(*seq);

            for stmt_or_decl in slice {
                if stmt_or_decl.tag == STATEMENT_OR_DECLARATION_DECLARATION {
                    let decl = unsafe { *stmt_or_decl.data.declaration };
                    if decl.tag == DECLARATION_LEXICAL_DECLARATION {
                        decls.push(OwnedParseNode::Declaration(decl));
                    }
                }
            }

            decls
        }
        _ => panic!(
            "Unsupported node type for top_level_lexically_scoped_decls: {:?}",
            node
        ),
    }
}

pub fn top_level_lexically_scoped_decls_owned(node: &OwnedParseNode) -> Vec<OwnedParseNode> {
    match node {
        OwnedParseNode::StatementOrDeclList(seq) => {
            let mut decls = Vec::new();
            let slice = collect_seq(seq);

            for stmt_or_decl in slice {
                if stmt_or_decl.tag == STATEMENT_OR_DECLARATION_DECLARATION {
                    let decl = unsafe { *stmt_or_decl.data.declaration };
                    if decl.tag == DECLARATION_LEXICAL_DECLARATION {
                        decls.push(OwnedParseNode::Declaration(decl));
                    }
                }
            }

            decls
        }
        _ => panic!(
            "Unsupported node type for top_level_lexically_scoped_decls_owned: {:?}",
            node
        ),
    }
}
