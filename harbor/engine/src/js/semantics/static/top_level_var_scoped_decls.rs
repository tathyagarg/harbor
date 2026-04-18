use crate::js::{
    collect_seq,
    semantics::r#static::{OwnedParseNode, ParseNode, StaticSemantics},
    stmt::{
        DECLARATION_ASYNC_FUNCTION_DECLARATION, DECLARATION_ASYNC_GENERATOR_DECLARATION,
        DECLARATION_FUNCTION_DECLARATION, DECLARATION_GENERATOR_DECLARATION,
        DECLARATION_LEXICAL_DECLARATION, Declaration, HoistableDeclaration,
        STATEMENT_OR_DECLARATION_DECLARATION, STATEMENT_OR_DECLARATION_STATEMENT,
        SeqStatementOrDeclaration, StatementOrDeclaration,
    },
};

fn top_level_var_scoped_decls_stmt_or_decl_list(
    stmt_or_decl_list: &SeqStatementOrDeclaration,
) -> Vec<OwnedParseNode> {
    let mut decls = Vec::new();
    let slice = collect_seq(stmt_or_decl_list);

    for stmt_or_decl in slice {
        decls.extend(top_level_var_scoped_decls_stmt_or_decl(&stmt_or_decl));
    }

    decls
}

fn top_level_var_scoped_decls_stmt_or_decl(
    stmt_or_decl: &StatementOrDeclaration,
) -> Vec<OwnedParseNode> {
    match stmt_or_decl.tag {
        STATEMENT_OR_DECLARATION_STATEMENT => {
            let statement = unsafe { *stmt_or_decl.data.statement };
            let parse_node = ParseNode::Statement(&statement);

            parse_node.var_scoped_declarations()
        }
        STATEMENT_OR_DECLARATION_DECLARATION => {
            let declaration = unsafe { *stmt_or_decl.data.declaration };
            if declaration.tag == DECLARATION_LEXICAL_DECLARATION {
                Vec::new()
            } else {
                top_level_var_scoped_decls_declaration(&declaration)
            }
        }
        _ => unreachable!(
            "Unexpected statement or declaration tag: {}",
            stmt_or_decl.tag
        ),
    }
}

fn top_level_var_scoped_decls_declaration(decl: &Declaration) -> Vec<OwnedParseNode> {
    match decl.tag {
        DECLARATION_LEXICAL_DECLARATION => vec![],
        DECLARATION_FUNCTION_DECLARATION => {
            top_level_var_scoped_decls_hoistable_decl(&unsafe { *decl.data.function })
        }
        DECLARATION_GENERATOR_DECLARATION => {
            top_level_var_scoped_decls_hoistable_decl(&unsafe { *decl.data.generator })
        }
        DECLARATION_ASYNC_FUNCTION_DECLARATION => {
            top_level_var_scoped_decls_hoistable_decl(&unsafe { *decl.data.async_function })
        }
        DECLARATION_ASYNC_GENERATOR_DECLARATION => {
            top_level_var_scoped_decls_hoistable_decl(&unsafe { *decl.data.async_generator })
        }
        _ => unreachable!(
            "Unexpected declaration tag in top_level_var_scoped_decls: {}",
            decl.tag,
        ),
    }
}

fn top_level_var_scoped_decls_hoistable_decl(
    hoistable: &HoistableDeclaration,
) -> Vec<OwnedParseNode> {
    vec![OwnedParseNode::HoistabeDeclaration(*hoistable)]
}

pub fn top_level_var_scoped_decls(target: &ParseNode) -> Vec<OwnedParseNode> {
    match target {
        ParseNode::StatementOrDeclList(stmt_or_decl_list) => {
            top_level_var_scoped_decls_stmt_or_decl_list(stmt_or_decl_list)
        }
        ParseNode::StatmentOrDeclaration(stmt_or_decl) => {
            top_level_var_scoped_decls_stmt_or_decl(stmt_or_decl)
        }
        ParseNode::Declaration(decl) => top_level_var_scoped_decls_declaration(decl),
        ParseNode::HoistabeDeclaration(hoistable) => {
            top_level_var_scoped_decls_hoistable_decl(hoistable)
        }
        _ => todo!(),
    }
}

pub fn top_level_var_scoped_decls_owned(target: &OwnedParseNode) -> Vec<OwnedParseNode> {
    match target {
        OwnedParseNode::StatementOrDeclList(stmt_or_decl_list) => {
            top_level_var_scoped_decls_stmt_or_decl_list(stmt_or_decl_list)
        }
        OwnedParseNode::StatmentOrDeclaration(stmt_or_decl) => {
            top_level_var_scoped_decls_stmt_or_decl(stmt_or_decl)
        }
        OwnedParseNode::Declaration(decl) => top_level_var_scoped_decls_declaration(decl),
        OwnedParseNode::HoistabeDeclaration(hoistable) => {
            top_level_var_scoped_decls_hoistable_decl(hoistable)
        }
        _ => todo!(),
    }
}
