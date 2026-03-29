use crate::js::{
    expr::{PRIMARY_EXPR_ARRAY, PRIMARY_EXPR_LITERAL, PrimaryExpression},
    values::ReferenceOrValue,
};

pub mod literals {
    use crate::js::{
        expr::{LITERAL_BOOLEAN, LITERAL_NULL, LITERAL_NUMBER, LITERAL_STRING, Literal},
        values::{ReferenceOrValue, Value, number::Number},
        zs_to_js_string,
    };

    pub fn evaluate(value: Literal) -> ReferenceOrValue {
        ReferenceOrValue::Value(match value.tag {
            LITERAL_NULL => Value::Null,
            LITERAL_BOOLEAN => Value::Boolean(unsafe { value.data.boolean }),
            LITERAL_NUMBER => {
                let num = unsafe { *value.data.numeric };
                Value::Number(Number(num.value))
            }
            LITERAL_STRING => {
                let s = unsafe { *value.data.string };
                Value::String(zs_to_js_string(s))
            }
            _ => unreachable!("Unknown literal tag: {}", value.tag),
        })
    }
}

pub mod arrays {
    use crate::js::{
        collect_seq,
        expr::{
            ARRAY_ELEMENT_ELISION, ARRAY_ELEMENT_EXPRESSION, ARRAY_ELEMENT_SPREAD, ArrayLiteral,
        },
        operations::set,
        types::completion_record::{CRKAbrupt, CompletionRecord, CompletionRecordError},
        values::{
            ReferenceOrValue, Value,
            number::Number,
            object::{ArrayObject, Object, PropertyKey},
        },
    };

    pub fn array_acculumation(
        array: &mut ArrayObject,
        array_syntax: ArrayLiteral,
        mut next_index: usize,
    ) -> Result<CompletionRecord<usize>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
        let elements = collect_seq(array_syntax.elements);
        let mut obj = Object::Array(array.clone());

        for elem in elements.iter() {
            match elem.tag {
                ARRAY_ELEMENT_ELISION => {
                    let new_len = next_index + 1;
                    let res = set(
                        &mut obj,
                        &PropertyKey::from("length"),
                        &Value::Number(Number(new_len as f64)),
                        true,
                    );

                    if let Err(e) = res {
                        return Err(CompletionRecord {
                            kind: CRKAbrupt::Throw,
                            value: e.unwrapped().clone(),
                            target: None,
                        });
                    }

                    next_index = new_len;
                }
                ARRAY_ELEMENT_EXPRESSION => {
                    todo!()
                }
                ARRAY_ELEMENT_SPREAD => {}
                _ => unreachable!("Unknown array element tag: {}", elem.tag),
            }
        }

        todo!()
    }

    pub fn evaluate(_array: ArrayLiteral) -> ReferenceOrValue {
        todo!("Array literal evaluation")
    }
}

pub fn evaluate(primary: PrimaryExpression) -> ReferenceOrValue {
    match primary.tag {
        PRIMARY_EXPR_LITERAL => {
            let literal_data = unsafe { *primary.data.literal };
            return literals::evaluate(literal_data);
        }
        PRIMARY_EXPR_ARRAY => {
            let array_data = unsafe { *primary.data.array };
            return arrays::evaluate(array_data);
        }
        _ => todo!("Implement evaluation of primaries of tag {}", primary.tag),
    }
}
