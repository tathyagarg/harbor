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
