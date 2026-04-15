use crate::js::{
    expr::Expression,
    semantics::expressions::EvaluateExpressionTag,
    stmt::{
        BlockStatement, DECLARATION_LEXICAL_DECLARATION, IfStatement, LexicalDeclaration,
        STATEMENT_BLOCK_STATEMENT, STATEMENT_EXPR_STATEMENT, STATEMENT_IF_STATEMENT,
        STATEMENT_OR_DECLARATION_DECLARATION, STATEMENT_OR_DECLARATION_STATEMENT, Statement,
        StatementOrDeclaration,
    },
    values::{ReferenceOrValue, Value},
};

pub mod block;
pub mod declarations;
pub mod if_stmt;

pub enum EvaluateStatementTag {
    LexicalDeclaration(LexicalDeclaration),
    Expression(Expression),
    IfStatement(IfStatement),
    BlockStatement(BlockStatement),

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
            _ => unimplemented!(
                "Only expression statements are implemented in statement_evaluate, not {:?}",
                stmt.tag
            ),
        },
    }
}
