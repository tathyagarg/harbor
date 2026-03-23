pub mod literals {
    use crate::js::{
        expr::{LITERAL_BOOLEAN, LITERAL_NULL, LITERAL_NUMBER, LITERAL_STRING, Literal},
        values::{Value, number::Number},
        zs_to_js_string,
    };

    pub fn evaluate(value: Literal) -> Value {
        match value.tag {
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
        }
    }
}

pub mod arrays {
    use crate::js::{
        collect_seq,
        expr::{
            ARRAY_ELEMENT_ELISION, ARRAY_ELEMENT_EXPRESSION, ARRAY_ELEMENT_SPREAD, ArrayElement,
            ArrayLiteral,
        },
        types::completion_record::{CRK, CRKAbrupt, CompletionRecord},
        values::object::{ArrayObject, Object},
    };

    pub fn array_acculumation(
        array: &mut ArrayObject,
        array_syntax: ArrayLiteral,
        mut next_index: usize,
    ) -> Result<CompletionRecord<usize>, CompletionRecord<(), CRKAbrupt>> {
        let elements = collect_seq(array_syntax.elements);
        let mut obj = Object::Array(array.clone());

        for elem in elements.iter() {
            match elem.tag {
                ARRAY_ELEMENT_ELISION => {
                    let new_len = next_index + 1;
                }
                ARRAY_ELEMENT_EXPRESSION => {}
                ARRAY_ELEMENT_SPREAD => {}
                _ => unreachable!("Unknown array element tag: {}", elem.tag),
            }
        }

        todo!()
    }

    // pub fn evaluate(array: ArrayLiteral, proto: Object) -> Result<CompletionRecord<>> {

    // }
}
