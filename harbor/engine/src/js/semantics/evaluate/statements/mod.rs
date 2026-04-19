use crate::js::{
    behaviours::functions::function_declaration_instantiation,
    collect_seq,
    executable::context::resolve_binding,
    expr::Expression,
    semantics::{evaluate::expressions::EvaluateExpressionTag, r#static::string_value},
    stmt::{
        BlockStatement, DECLARATION_FUNCTION_DECLARATION, DECLARATION_LEXICAL_DECLARATION,
        IfStatement, LexicalDeclaration, STATEMENT_BLOCK_STATEMENT, STATEMENT_EXPR_STATEMENT,
        STATEMENT_IF_STATEMENT, STATEMENT_OR_DECLARATION_DECLARATION,
        STATEMENT_OR_DECLARATION_STATEMENT, STATEMENT_VAR_STATEMENT, Script, Statement,
        StatementOrDeclaration,
    },
    types::completion_record::{CRKReturn, CRKThrow, CompletionRecord},
    values::{ReferenceOrValue, Value, object::FunctionObject, reference::put_value},
};

pub mod block;
pub mod declarations;
pub mod if_stmt;

pub enum EvaluateStatementTag {
    LexicalDeclaration(LexicalDeclaration),
    Expression(Expression),
    IfStatement(IfStatement),
    BlockStatement(BlockStatement),
    Script(Script),

    Statement(Statement),
}

pub fn statement_or_declaration_evaluate(val: &StatementOrDeclaration) -> ReferenceOrValue {
    match val.tag {
        STATEMENT_OR_DECLARATION_DECLARATION => {
            let decl = unsafe { *val.data.declaration };

            match decl.tag {
                DECLARATION_LEXICAL_DECLARATION => {
                    let lex_decl = unsafe { *decl.data.lex_decl };
                    statement_evaluate(&EvaluateStatementTag::LexicalDeclaration(lex_decl))
                }
                DECLARATION_FUNCTION_DECLARATION => {
                    return ReferenceOrValue::Value(Value::Undefined);
                }
                _ => unimplemented!(
                    "Only lexical declarations are implemented in statement_or_declaration_evaluate"
                ),
            }
        }
        STATEMENT_OR_DECLARATION_STATEMENT => {
            let stmt = unsafe { *val.data.statement };
            statement_evaluate(&EvaluateStatementTag::Statement(stmt))
        }
        _ => unimplemented!(
            "Only declaration and statement are implemented in statement_or_declaration_evaluate"
        ),
    }
}

pub fn script_evaluate(script: &Script) -> ReferenceOrValue {
    block::statement_list_evaluate(&script.body)
}

pub fn statement_evaluate(tag: &EvaluateStatementTag) -> ReferenceOrValue {
    match tag {
        EvaluateStatementTag::LexicalDeclaration(decl) => {
            declarations::evaluate(decl);
            return ReferenceOrValue::Value(Value::Undefined);
        }
        EvaluateStatementTag::Expression(expr) => super::expressions::expression_evaluate(
            &EvaluateExpressionTag::Expression(expr.clone()),
        ),
        EvaluateStatementTag::IfStatement(stmt) => if_stmt::evaluate(stmt),
        EvaluateStatementTag::BlockStatement(stmt) => block::evaluate(stmt),
        EvaluateStatementTag::Script(script) => script_evaluate(script),

        EvaluateStatementTag::Statement(stmt) => match stmt.tag {
            STATEMENT_EXPR_STATEMENT => {
                let expr_stmt = unsafe { *stmt.data.expression };
                statement_evaluate(&EvaluateStatementTag::Expression(expr_stmt))
            }
            STATEMENT_IF_STATEMENT => {
                let if_stmt = unsafe { *stmt.data.if_stmt };
                statement_evaluate(&EvaluateStatementTag::IfStatement(if_stmt))
            }
            STATEMENT_BLOCK_STATEMENT => {
                let block_stmt = unsafe { *stmt.data.block };
                statement_evaluate(&EvaluateStatementTag::BlockStatement(block_stmt))
            }
            STATEMENT_VAR_STATEMENT => {
                let statement = unsafe { *stmt.data.var };
                let raw_decls = statement.bindings;
                let decls = collect_seq(&raw_decls);

                for decl in decls {
                    let raw_right = unsafe { *decl.initializer };
                    if !raw_right.has_value {
                        continue;
                    }

                    let right = unsafe { raw_right.value.value };

                    let binding_id = string_value(unsafe { *decl.name });
                    let lhs = resolve_binding(binding_id, None).unwrap().value;

                    let rhs = super::expressions::expression_evaluate(
                        &EvaluateExpressionTag::AssignmentExpression(right),
                    );
                    let value = rhs.get_value().unwrap().value;

                    put_value(&mut ReferenceOrValue::Reference(lhs), &value).unwrap();
                }

                return ReferenceOrValue::Value(Value::Undefined);
            }
            _ => unimplemented!(
                "Only expression statements are implemented in statement_evaluate, not {:?}",
                stmt.tag
            ),
        },
    }
}

pub fn evaluate_function_body(
    function: &FunctionObject,
    arguments: Vec<Value>,
) -> Result<CompletionRecord<Value, CRKReturn>, CompletionRecord<(), CRKThrow>> {
    function_declaration_instantiation(function, arguments)?;
    let res = statement_evaluate(&EvaluateStatementTag::BlockStatement(
        function.ecmascript_code,
    ));

    Ok(CompletionRecord {
        kind: CRKReturn,
        value: res
            .get_value()
            .ok()
            .map(|v| v.value)
            .unwrap_or(Value::Undefined),
        target: None,
    })
}
