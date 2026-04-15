use std::{cell::RefCell, rc::Rc};

use crate::js::{
    collect_seq,
    executable::{
        agent::running_execution_context,
        environment::{EnvironmentRecord, new_declarative_environment},
    },
    stmt::{BlockStatement, SeqStatementOrDeclaration},
    syntax::{bound_names_declaration, is_constant_decl, lexically_scope_declarations_stmt_lis},
    values::{ReferenceOrValue, Value},
};

pub fn evaluate(stmt: &BlockStatement) -> ReferenceOrValue {
    let old_env = running_execution_context()
        .unwrap()
        .borrow()
        .lexical_env()
        .unwrap();
    let block_env = Rc::new(RefCell::new(new_declarative_environment(Some(
        old_env.clone(),
    ))));

    block_declaration_instantiation(&stmt.body, block_env.clone());

    running_execution_context()
        .unwrap()
        .borrow_mut()
        .replace_lexical_env(block_env);

    for stmt_or_decl in collect_seq(&stmt.body) {
        super::statement_or_declaration_evaluate(&stmt_or_decl);
    }

    running_execution_context()
        .unwrap()
        .borrow_mut()
        .replace_lexical_env(old_env);

    ReferenceOrValue::Value(Value::Undefined)
}

fn block_declaration_instantiation(
    code: &SeqStatementOrDeclaration,
    env: Rc<RefCell<EnvironmentRecord>>,
) {
    let declarations = lexically_scope_declarations_stmt_lis(&code);

    for decl in declarations {
        for name in bound_names_declaration(&decl) {
            if is_constant_decl(&decl) {
                env.borrow_mut()
                    .create_immutable_binding(name.clone(), true)
                    .unwrap();
            } else {
                env.borrow_mut()
                    .create_mutable_binding(name.clone(), false)
                    .unwrap();
            }
        }
    }
}
