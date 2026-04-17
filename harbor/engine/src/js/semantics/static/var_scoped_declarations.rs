use crate::js::{
    collect_seq,
    semantics::r#static::ParseNode,
    stmt::{
        BlockStatement, LexicalBinding, STATEMENT_BLOCK_STATEMENT, STATEMENT_BREAK_STATEMENT,
        STATEMENT_CONTINUE_STATEMENT, STATEMENT_DEBUGGER_STATEMENT, STATEMENT_DO_WHILE,
        STATEMENT_EMPTY_STATEMENT, STATEMENT_EXPR_STATEMENT, STATEMENT_IF_STATEMENT,
        STATEMENT_OR_DECLARATION_DECLARATION, STATEMENT_OR_DECLARATION_STATEMENT,
        STATEMENT_RETURN_STATEMENT, STATEMENT_THROW_STATEMENT, STATEMENT_VAR_STATEMENT,
        STATEMENT_WHILE, Statement,
    },
};

// pub enum VarScopedDeclarations<'a> {
//     Statement(&'a Statement),
//     BlockStatement(&'a BlockStatement),
// }

fn var_scoped_declarations_statement(statement: &Statement) -> Vec<LexicalBinding> {
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
            var_scoped_declarations_block_statement(&block_stmt)
        }
        STATEMENT_VAR_STATEMENT => {
            let var_stmt = unsafe { *statement.data.var };
            let declarations = collect_seq(&var_stmt.bindings);

            declarations
        }
        STATEMENT_IF_STATEMENT => {
            let if_stmt = unsafe { *statement.data.if_stmt };
            let mut decls = Vec::new();

            decls.extend(var_scoped_declarations_statement(&unsafe {
                *if_stmt.consequent
            }));

            let alternate = unsafe { *if_stmt.alternate };
            if alternate.has_value {
                decls.extend(var_scoped_declarations_statement(&unsafe {
                    alternate.value.value
                }));
            }

            decls
        }
        STATEMENT_DO_WHILE => {
            let do_while_stmt = unsafe { *statement.data.do_while };
            var_scoped_declarations_statement(&unsafe { *do_while_stmt.body })
        }
        STATEMENT_WHILE => {
            let while_stmt = unsafe { *statement.data.while_ };
            var_scoped_declarations_statement(&unsafe { *while_stmt.body })
        }
        _ => unreachable!("Unexpected statement tag: {}", statement.tag),
    }
}

fn var_scoped_declarations_block_statement(block_stmt: &BlockStatement) -> Vec<LexicalBinding> {
    let mut decls = Vec::new();
    let slice = crate::js::collect_seq(&block_stmt.body);

    for stmt in slice {
        match stmt.tag {
            STATEMENT_OR_DECLARATION_DECLARATION => {}
            STATEMENT_OR_DECLARATION_STATEMENT => {
                let statement = unsafe { *stmt.data.statement };
                decls.extend(var_scoped_declarations_statement(&statement));
            }
            _ => unreachable!("Unexpected statement or declaration tag: {}", stmt.tag),
        }
    }

    decls
}

pub fn var_scoped_declarations(target: ParseNode) -> Vec<LexicalBinding> {
    match target {
        ParseNode::Statement(stmt) => var_scoped_declarations_statement(stmt),
        ParseNode::BlockStatement(block_stmt) => {
            var_scoped_declarations_block_statement(block_stmt)
        }
        _ => unimplemented!(
            "var_scoped_declarations not implemented for target: {:?}",
            target
        ),
    }
}
