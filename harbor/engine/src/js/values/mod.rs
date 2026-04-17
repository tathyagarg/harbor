use std::rc::Rc;

use crate::js::{
    types::completion_record::{CRKAbrupt, CompletionRecord, CompletionRecordError},
    values::{
        number::Number,
        object::Object,
        reference::{Reference, get_value},
        string::JsString,
        symbol::Symbol,
    },
};

pub mod number;
pub mod object;

pub mod reference;

#[derive(Clone, Debug)]
pub enum ReferenceOrValue {
    Reference(Reference),
    Value(Value),
}

impl ReferenceOrValue {
    pub fn get_value(
        &self,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
        get_value(self)
    }
}

pub mod string {
    use std::{ops::Add, str::FromStr};

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct JsString(pub Vec<u16>);

    impl JsString {
        pub fn concat(&self, other: &JsString) -> JsString {
            let mut combined = self.0.clone();
            combined.extend(other.0.iter());
            JsString(combined)
        }

        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub fn code_point_at(&self, index: usize) -> Option<u16> {
            self.0.get(index).cloned()
        }
    }

    impl From<JsString> for String {
        fn from(js_str: JsString) -> Self {
            String::from_utf16(&js_str.0).unwrap_or_default()
        }
    }

    impl FromStr for JsString {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(JsString(s.encode_utf16().collect()))
        }
    }

    impl Add for JsString {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            let mut combined = self.0;
            combined.extend(rhs.0);
            JsString(combined)
        }
    }

    impl PartialEq<&str> for JsString {
        fn eq(&self, other: &&str) -> bool {
            let other_utf16: Vec<u16> = other.encode_utf16().collect();
            self.0 == other_utf16
        }

        fn ne(&self, other: &&str) -> bool {
            !self.eq(other)
        }
    }

    pub fn string_index_of(
        string: &JsString,
        search_value: &JsString,
        from_index: usize,
    ) -> Option<usize> {
        let len = string.0.len();

        if search_value.0.is_empty() && from_index <= len {
            return Some(from_index);
        }

        let search_len = search_value.0.len();

        if search_len + from_index > len {
            return None;
        }

        for i in from_index..=len - search_len {
            let candidate = &string.0[i..i + search_len];
            if candidate == search_value.0.as_slice() {
                return Some(i);
            }
        }

        None
    }

    pub fn string_last_index_of(
        string: &JsString,
        search_value: &JsString,
        from_index: usize,
    ) -> Option<usize> {
        let search_len = search_value.0.len();

        for i in (0..=from_index).rev() {
            let candidate = &string.0[i..i + search_len];
            if candidate == search_value.0.as_slice() {
                return Some(i);
            }
        }

        None
    }

    pub fn _equals_raw(string1: &JsString, string2: &str) -> bool {
        let string2_utf16: Vec<u16> = string2.encode_utf16().collect();
        string1.0 == string2_utf16
    }
}

pub mod symbol {
    use std::{str::FromStr, sync::LazyLock};

    use crate::js::values::string::JsString;

    pub type SymbolId = u64;

    pub const SYMBOL_TO_PRIMITIVE: SymbolId = 0;

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct Symbol {
        pub id: SymbolId,
        pub description: Option<JsString>,
    }

    pub static SYMBOL_ITERATOR: LazyLock<Symbol> = LazyLock::new(|| Symbol {
        id: 0,
        description: Some(JsString::from_str("Symbol.iterator").unwrap()),
    });
}

#[derive(Clone, Debug)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    String(JsString),
    Symbol(Symbol),
    Number(Number),
    BigInt(()),
    Object(Object),

    InternalFunction(Rc<fn(Vec<Value>) -> Value>),
}

impl Value {
    pub fn empty() -> Self {
        Value::Undefined
    }

    pub fn unwrap_bool(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn unwrap_number(&self) -> Option<Number> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn unwrap_object(&self) -> Option<Object> {
        if let Value::Object(o) = self {
            Some(o.clone())
        } else {
            None
        }
    }

    pub fn unwrap_string(&self) -> Option<JsString> {
        if let Value::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn unwrap_symbol(&self) -> Option<Symbol> {
        if let Value::Symbol(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn is_property_key(&self) -> bool {
        matches!(self, Value::String(_) | Value::Symbol(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, Value::Undefined)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
}
