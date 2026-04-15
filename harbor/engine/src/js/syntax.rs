use crate::js::{
    collect_seq,
    semantics::expressions::identifier::string_value,
    stmt::{
        DECLARATION_LEXICAL_DECLARATION, Declaration, LexicalDeclaration,
        STATEMENT_OR_DECLARATION_DECLARATION, STATEMENT_OR_DECLARATION_STATEMENT, Script,
        SeqStatementOrDeclaration, Statement,
    },
    values::string::JsString,
};

fn bound_names_lexical_declaration(lex_decl: &LexicalDeclaration) -> Vec<JsString> {
    let bindings = collect_seq(&lex_decl.bindings);

    let mut names = Vec::new();

    for binding in bindings {
        let identifier = unsafe { *binding.name };
        let name = string_value(identifier);

        names.push(name);
    }

    names
}

pub fn bound_names_declaration(declaration: &Declaration) -> Vec<JsString> {
    match declaration.tag {
        DECLARATION_LEXICAL_DECLARATION => {
            println!("Found lexical declaration");
            let lexical_decl = unsafe { *declaration.data.lex_decl };
            println!("Lexical declaration: {}", lexical_decl);
            bound_names_lexical_declaration(&lexical_decl)
        }
        _ => vec![],
    }
}

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

pub fn lexically_scoped_declarations(script: &Script) -> Vec<Declaration> {
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

pub fn var_declared_names_script(script: &Script) -> Vec<JsString> {
    return top_level_var_declared_names_script(&script.body);
}

/// TODO: I haven't implemented parsing of var declarations
pub fn var_declared_names_statement(stmt: &Statement) -> Vec<JsString> {
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

                let decl_names = bound_names_declaration(decl);
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
