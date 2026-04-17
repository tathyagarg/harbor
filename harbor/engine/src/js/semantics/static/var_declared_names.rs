use crate::js::{
    collect_seq,
    semantics::r#static::{BoundNames, bound_names},
    stmt::{
        BlockStatement, DECLARATION_ASYNC_FUNCTION_DECLARATION,
        DECLARATION_ASYNC_GENERATOR_DECLARATION, DECLARATION_FUNCTION_DECLARATION,
        DECLARATION_GENERATOR_DECLARATION, IfStatement, STATEMENT_BLOCK_STATEMENT,
        STATEMENT_BREAK_STATEMENT, STATEMENT_CONTINUE_STATEMENT, STATEMENT_DEBUGGER_STATEMENT,
        STATEMENT_EMPTY_STATEMENT, STATEMENT_EXPR_STATEMENT, STATEMENT_OR_DECLARATION_DECLARATION,
        STATEMENT_OR_DECLARATION_STATEMENT, STATEMENT_RETURN_STATEMENT, STATEMENT_THROW_STATEMENT,
        Script, SeqStatementOrDeclaration, Statement, StatementOrDeclaration, WhileStatement,
    },
    values::string::JsString,
};

pub enum VarDeclaredNames<'a> {
    Script(&'a Script),
    Statement(&'a Statement),
    StatmentOrDeclaration(&'a StatementOrDeclaration),
    BlockStatement(&'a BlockStatement),
    IfStatement(&'a IfStatement),
    WhileStatement(&'a WhileStatement),
}

fn var_declared_names_script(script: &Script) -> Vec<JsString> {
    return top_level_var_declared_names(&script.body);
}

fn var_declared_names_statement_or_decl(stmt_or_decl: &StatementOrDeclaration) -> Vec<JsString> {
    match stmt_or_decl.tag {
        STATEMENT_OR_DECLARATION_STATEMENT => {
            let statement = unsafe { *stmt_or_decl.data.statement };
            var_declared_names_statement(&statement)
        }
        STATEMENT_OR_DECLARATION_DECLARATION => vec![],
        _ => unreachable!(
            "Unexpected statement or declaration tag: {}",
            stmt_or_decl.tag
        ),
    }
}

fn var_declared_names_block_statement(block_stmt: &BlockStatement) -> Vec<JsString> {
    let mut names = Vec::new();
    let slice = collect_seq(&block_stmt.body);

    for stmt in slice {
        names.extend(var_declared_names_statement_or_decl(&stmt));
    }

    names
}

fn var_declared_names_statement(statement: &Statement) -> Vec<JsString> {
    match statement.tag {
        STATEMENT_EMPTY_STATEMENT
        | STATEMENT_EXPR_STATEMENT
        | STATEMENT_CONTINUE_STATEMENT
        | STATEMENT_BREAK_STATEMENT
        | STATEMENT_RETURN_STATEMENT
        | STATEMENT_THROW_STATEMENT
        | STATEMENT_DEBUGGER_STATEMENT => vec![],
        STATEMENT_BLOCK_STATEMENT => {
            let block_stmt = unsafe { *statement.data.block };
            var_declared_names_block_statement(&block_stmt)
        }
        _ => todo!("var_declared_names for statement tag: {}", statement.tag),
    }
}

fn var_declared_names_while(while_stmt: &WhileStatement) -> Vec<JsString> {
    let mut names = Vec::new();

    let body = unsafe { *while_stmt.body };
    names.extend(var_declared_names_statement(&body));

    names
}

fn var_declared_names_if_statement(stmt: &IfStatement) -> Vec<JsString> {
    let mut names = Vec::new();

    let names1 = var_declared_names_statement(&unsafe { *stmt.consequent });
    names.extend(names1);

    let alternate = unsafe { *stmt.alternate };
    if alternate.has_value {
        let names2 = var_declared_names_statement(&unsafe { alternate.value.value });
        names.extend(names2);
    }

    names
}

pub fn top_level_var_declared_names(list: &SeqStatementOrDeclaration) -> Vec<JsString> {
    let mut names = Vec::<JsString>::new();
    let sice = collect_seq(list);

    for stmt in sice {
        match stmt.tag {
            STATEMENT_OR_DECLARATION_DECLARATION => {
                let decl = unsafe { *stmt.data.declaration };
                if decl.tag == DECLARATION_ASYNC_FUNCTION_DECLARATION
                    || decl.tag == DECLARATION_ASYNC_GENERATOR_DECLARATION
                    || decl.tag == DECLARATION_FUNCTION_DECLARATION
                    || decl.tag == DECLARATION_GENERATOR_DECLARATION
                {
                    let hoistable_decl = match decl.tag {
                        DECLARATION_ASYNC_FUNCTION_DECLARATION => unsafe {
                            *decl.data.async_function
                        },
                        DECLARATION_ASYNC_GENERATOR_DECLARATION => unsafe {
                            *decl.data.async_generator
                        },
                        DECLARATION_FUNCTION_DECLARATION => unsafe { *decl.data.function },
                        DECLARATION_GENERATOR_DECLARATION => unsafe { *decl.data.generator },
                        _ => unreachable!(),
                    };

                    names.extend(bound_names(BoundNames::HoistabeDeclaration(
                        &hoistable_decl,
                    )));
                }
            }
            STATEMENT_OR_DECLARATION_STATEMENT => {
                let statement = unsafe { *stmt.data.statement };
                names.extend(var_declared_names(VarDeclaredNames::Statement(&statement)));
            }
            _ => unreachable!("Unexpected statement or declaration tag: {}", stmt.tag),
        }
    }

    names
}

pub fn var_declared_names(target: VarDeclaredNames) -> Vec<JsString> {
    match target {
        VarDeclaredNames::Script(script) => var_declared_names_script(&script),
        VarDeclaredNames::Statement(stmt) => var_declared_names_statement(&stmt),
        VarDeclaredNames::StatmentOrDeclaration(stmt_or_decl) => {
            var_declared_names_statement_or_decl(&stmt_or_decl)
        }
        VarDeclaredNames::BlockStatement(block_stmt) => {
            var_declared_names_block_statement(&block_stmt)
        }
        VarDeclaredNames::IfStatement(if_stmt) => var_declared_names_if_statement(&if_stmt),
        VarDeclaredNames::WhileStatement(while_stmt) => var_declared_names_while(&while_stmt),
    }
}
