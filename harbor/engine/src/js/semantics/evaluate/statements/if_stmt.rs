use crate::js::{
    operations::to_boolean,
    semantics::evaluate::{expressions::EvaluateExpressionTag, statements::EvaluateStatementTag},
    stmt::IfStatement,
    values::{ReferenceOrValue, Value},
};

pub fn evaluate(stmt: &IfStatement) -> ReferenceOrValue {
    let test = unsafe { *stmt.test };
    let expr_ref =
        super::super::expressions::expression_evaluate(&EvaluateExpressionTag::Expression(test));
    let expr_val = to_boolean(&expr_ref.get_value().unwrap().value);

    if expr_val {
        let cons = unsafe { *stmt.consequent };
        super::statement_evaluate(&EvaluateStatementTag::Statement(cons));
    } else {
        let alt = unsafe { *stmt.alternate };
        if alt.has_value {
            let value = unsafe { alt.value.value };
            super::statement_evaluate(&EvaluateStatementTag::Statement(value));
        }
    };

    ReferenceOrValue::Value(Value::Undefined)
}
