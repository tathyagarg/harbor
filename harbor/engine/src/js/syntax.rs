use crate::js::{
    collect_seq,
    semantics::r#static::{ParseNode, bound_names},
    stmt::{
        DECLARATION_LEXICAL_DECLARATION, Declaration, STATEMENT_OR_DECLARATION_DECLARATION,
        STATEMENT_OR_DECLARATION_STATEMENT, Script, SeqStatementOrDeclaration, Statement,
    },
    values::string::JsString,
};

pub fn is_constant_decl(declaration: &Declaration) -> bool {
    match declaration.tag {
        DECLARATION_LEXICAL_DECLARATION => {
            let lexical_decl = unsafe { *declaration.data.lex_decl };
            lexical_decl.is_const
        }
        _ => false,
    }
}

pub fn lexically_declared_names_script(script: &Script) -> Vec<JsString> {
    return top_level_lexically_declared_names_script(&script.body);
}

pub fn lexically_scope_declarations_stmt_lis(list: &SeqStatementOrDeclaration) -> Vec<Declaration> {
    let mut decls = Vec::new();
    let slice = collect_seq(list);

    for stmt in slice {
        match stmt.tag {
            STATEMENT_OR_DECLARATION_STATEMENT => {}
            STATEMENT_OR_DECLARATION_DECLARATION => {
                let decl = unsafe { stmt.data.declaration };
                let decl = unsafe { &*decl };

                if decl.tag == DECLARATION_LEXICAL_DECLARATION {
                    decls.push(*decl);
                }
            }
            _ => unreachable!(),
        }
    }

    decls
}

pub fn lexically_scoped_declarations_script(script: &Script) -> Vec<Declaration> {
    let mut decls = Vec::new();
    let slice = collect_seq(&script.body);

    for stmt in slice {
        match stmt.tag {
            STATEMENT_OR_DECLARATION_STATEMENT => {}
            STATEMENT_OR_DECLARATION_DECLARATION => {
                let decl = unsafe { stmt.data.declaration };
                let decl = unsafe { &*decl };

                if decl.tag == DECLARATION_LEXICAL_DECLARATION {
                    decls.push(*decl);
                }
            }
            _ => unreachable!(),
        }
    }

    decls
}

/// TODO: I haven't implemented parsing of var declarations
pub fn var_declared_names_statement(_stmt: &Statement) -> Vec<JsString> {
    vec![]

    // let mut names = Vec::new();

    // match stmt.tag {
    //     STATEMENT_VAR_STATEMENT => {
    //         let var_stmt = unsafe { *stmt.data.var };
    //         let declarations = collect_seq(&var_stmt.declarations);

    //         for decl in declarations {
    //             let identifier = unsafe { *decl.bindings };
    //             let name = string_value(identifier);

    //             names.push(name);
    //         }

    //         names
    //     }
    //     _ => vec![],
    // }
}

pub fn top_level_lexically_declared_names_script(
    statements: &SeqStatementOrDeclaration,
) -> Vec<JsString> {
    let mut names = Vec::new();
    let slice = collect_seq(statements);

    for stmt in slice {
        match stmt.tag {
            STATEMENT_OR_DECLARATION_STATEMENT => {}
            STATEMENT_OR_DECLARATION_DECLARATION => {
                let decl = unsafe { stmt.data.declaration };
                let decl = unsafe { &*decl };

                let decl_names = bound_names(ParseNode::Declaration(decl));
                names.extend(decl_names);
            }
            _ => unreachable!(),
        }
    }

    names
}

pub fn top_level_var_declared_names_script(
    statements: &SeqStatementOrDeclaration,
) -> Vec<JsString> {
    let mut names = Vec::new();
    let slice = collect_seq(statements);

    for stmt in slice {
        match stmt.tag {
            STATEMENT_OR_DECLARATION_STATEMENT => {
                let stmt = unsafe { *stmt.data.statement };

                names.extend(var_declared_names_statement(&stmt));
            }
            STATEMENT_OR_DECLARATION_DECLARATION => {}
            _ => unreachable!(),
        }
    }

    names
}
