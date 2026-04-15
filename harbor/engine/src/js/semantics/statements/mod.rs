use crate::js::{
    expr::Expression,
    semantics::expressions::EvaluateExpressionTag,
    stmt::{
        DECLARATION_LEXICAL_DECLARATION, LexicalDeclaration, STATEMENT_EXPR_STATEMENT,
        STATEMENT_OR_DECLARATION_DECLARATION, STATEMENT_OR_DECLARATION_STATEMENT,
        StatementOrDeclaration,
    },
    values::{ReferenceOrValue, Value},
};

pub mod declarations;

pub enum EvaluateStatementTag {
    LexicalDeclaration(LexicalDeclaration),
    Expression(Expression),
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

            match stmt.tag {
                STATEMENT_EXPR_STATEMENT => {
                    let expr_stmt = unsafe { *stmt.data.expression };
                    statement_evaluate(&EvaluateStatementTag::Expression(expr_stmt))
                }
                _ => unimplemented!(
                    "Only expression statements are implemented in statement_or_declaration_evaluate"
                ),
            }
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
    }
}
