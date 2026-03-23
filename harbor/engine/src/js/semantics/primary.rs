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
        expr::ArrayLiteral,
        types::completion_record::{CRK, CRKAbrupt, CRKNormal, CompletionRecord},
        values::object::{ArrayObject, Object},
    };

    pub fn array_acculumation(
        array: ArrayObject,
        next_index: usize,
    ) -> Result<CompletionRecord<usize, CRKNormal>, CompletionRecord<(), CRKAbrupt>> {
        todo!()
    }

    // pub fn evaluate(array: ArrayLiteral, proto: Object) -> Result<CompletionRecord<>> {

    // }
}
