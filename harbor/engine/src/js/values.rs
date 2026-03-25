use crate::js::{
    executable::environment::EnvironmentRecord,
    values::{number::Number, object::Object, string::JsString, symbol::Symbol},
};

pub mod string {
    use std::{ops::Add, str::FromStr};

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct JsString(pub Vec<u16>);

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
    use crate::js::values::string::JsString;

    pub type SymbolId = u64;

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct Symbol {
        pub id: SymbolId,
        pub description: Option<JsString>,
    }
}

pub mod number {
    use std::{ops::Add, str::FromStr};

    use crate::js::{
        operations::{to_int32, to_uint32},
        values::{Value, string::JsString},
    };

    pub enum BitwiseOp {
        And,
        Xor,
        Or,
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Number(pub f64);

    impl Add<f64> for Number {
        type Output = Self;

        fn add(self, rhs: f64) -> Self::Output {
            Number(self.0 + rhs)
        }
    }

    impl Number {
        pub fn unary_minus(&self) -> Self {
            Number(if self.0.is_nan() { f64::NAN } else { -self.0 })
        }

        pub fn bitwise_not(&self) -> Self {
            let old_value = to_int32(Value::Number(*self)).unwrap().value.0 as i32;
            return Number((!old_value) as f64);
        }

        /// NOTE: This may not be spec-compliant
        pub fn exponentiate(&self, other: &Number) -> Self {
            Number(self.0.powf(other.0))
        }

        pub fn multiply(&self, other: &Number) -> Self {
            Number(self.0 * other.0)
        }

        pub fn divide(&self, other: &Number) -> Self {
            Number(self.0 / other.0)
        }

        pub fn remainder(&self, other: &Number) -> Self {
            Number(self.0 % other.0)
        }

        pub fn add(&self, other: &Number) -> Self {
            Number(self.0 + other.0)
        }

        pub fn subtract(&self, other: &Number) -> Self {
            Number(self.0 - other.0)
        }

        pub fn left_shift(&self, other: &Number) -> Self {
            let left = to_int32(Value::Number(*self)).unwrap().value.0 as i32;
            let right = to_uint32(Value::Number(*other)).unwrap().value.0 as u32;
            let shift_count = right % 32;

            Number(((left << shift_count) as i32) as f64)
        }

        pub fn signed_right_shift(&self, other: &Number) -> Self {
            let left = to_int32(Value::Number(*self)).unwrap().value.0 as i32;
            let right = to_uint32(Value::Number(*other)).unwrap().value.0 as u32;
            let shift_count = right % 32;

            Number((left >> shift_count) as f64)
        }

        pub fn unsigned_right_shift(&self, other: &Number) -> Self {
            let left = to_uint32(Value::Number(*self)).unwrap().value.0 as u32;
            let right = to_uint32(Value::Number(*other)).unwrap().value.0 as u32;
            let shift_count = right % 32;

            Number((left >> shift_count) as f64)
        }

        pub fn less_than(&self, other: &Number) -> bool {
            self.0 < other.0
        }

        pub fn equal(&self, other: &Number) -> bool {
            self.0 == other.0
        }

        pub fn same_value(&self, other: &Number) -> bool {
            if self.0.is_nan() && other.0.is_nan() {
                return true;
            }

            if self.0 == 0.0 && other.0 == 0.0 {
                return self.0.is_sign_negative() == other.0.is_sign_negative();
            }

            self.0 == other.0
        }

        pub fn same_value_zero(&self, other: &Number) -> bool {
            if self.0.is_nan() && other.0.is_nan() {
                return true;
            }

            self.0 == other.0
        }

        pub fn bitwise_and(&self, other: &Number) -> Self {
            number_bitwise_op(BitwiseOp::And, self, other)
        }

        pub fn bitwise_xor(&self, other: &Number) -> Self {
            number_bitwise_op(BitwiseOp::Xor, self, other)
        }

        pub fn bitwise_or(&self, other: &Number) -> Self {
            number_bitwise_op(BitwiseOp::Or, self, other)
        }

        /// NOTE: This may not be spec-compliant
        pub fn to_string(&self, radix: u8) -> JsString {
            if self.0.is_nan() {
                return JsString::from_str("NaN").unwrap();
            }

            if self.0 == 0.0 {
                return JsString::from_str("0").unwrap();
            }

            if self.0 < 0.0 {
                return JsString::from_str("-").unwrap() + Number(-self.0).to_string(radix);
            }

            if self.0.is_infinite() {
                return JsString::from_str("Infinity").unwrap();
            }

            return JsString::from_str(&self.0.to_string().to_uppercase()).unwrap();
        }
    }

    pub fn number_bitwise_op(op: BitwiseOp, left: &Number, right: &Number) -> Number {
        let left_int = to_int32(Value::Number(*left)).unwrap().value.0 as i32;
        let right_int = to_int32(Value::Number(*right)).unwrap().value.0 as i32;

        let result = match op {
            BitwiseOp::And => left_int & right_int,
            BitwiseOp::Xor => left_int ^ right_int,
            BitwiseOp::Or => left_int | right_int,
        };

        Number(result as f64)
    }
}

pub mod object {
    use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};

    use crate::js::{
        behaviours::{ordinary_get_own_property, ordinary_set, ordinary_set_prototype_of},
        values::{Value, number::Number, string::JsString, symbol::Symbol},
    };

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub enum PropertyKey {
        String(JsString),
        Symbol(Symbol),
    }

    impl PropertyKey {
        pub fn empty() -> Self {
            PropertyKey::String(JsString::from_str("").unwrap())
        }
    }

    impl From<String> for PropertyKey {
        fn from(value: String) -> Self {
            PropertyKey::String(JsString::from_str(&value).unwrap())
        }
    }

    impl From<&str> for PropertyKey {
        fn from(value: &str) -> Self {
            PropertyKey::String(JsString::from_str(value).unwrap())
        }
    }

    impl From<JsString> for PropertyKey {
        fn from(value: JsString) -> Self {
            PropertyKey::String(value)
        }
    }

    impl Into<JsString> for PropertyKey {
        fn into(self) -> JsString {
            match self {
                PropertyKey::String(s) => s,
                PropertyKey::Symbol(_) => panic!("Cannot convert Symbol to String"),
            }
        }
    }

    impl Into<JsString> for &PropertyKey {
        fn into(self) -> JsString {
            match self {
                PropertyKey::String(s) => s.clone(),
                PropertyKey::Symbol(_) => panic!("Cannot convert Symbol to String"),
            }
        }
    }

    impl PartialEq<&str> for PropertyKey {
        fn eq(&self, other: &&str) -> bool {
            match self {
                PropertyKey::String(s) => JsString::from_str(other).unwrap() == *s,
                PropertyKey::Symbol(_) => false,
            }
        }

        fn ne(&self, other: &&str) -> bool {
            !self.eq(other)
        }
    }

    #[derive(Debug, Clone)]
    pub enum PropertyDescriptor {
        Data {
            value: Value,
            writable: bool,
            enumerable: bool,
            configurable: bool,
        },
        Accessor {
            /// NOTE: Object or undefined
            get: Value,
            /// NOTE: Object or undefined
            set: Value,
            enumerable: bool,
            configurable: bool,
        },
        NonGeneric {
            fields: HashMap<String, Value>,
        },
    }

    impl PropertyDescriptor {
        pub fn fields(&self) -> Vec<String> {
            match self {
                PropertyDescriptor::Data { .. } => vec![
                    "value".to_string(),
                    "writable".to_string(),
                    "enumerable".to_string(),
                    "configurable".to_string(),
                ],
                PropertyDescriptor::Accessor { .. } => vec![
                    "get".to_string(),
                    "set".to_string(),
                    "enumerable".to_string(),
                    "configurable".to_string(),
                ],
                PropertyDescriptor::NonGeneric { fields } => fields.keys().cloned().collect(),
            }
        }

        pub fn data_descriptor(
            value: Value,
            writable: bool,
            enumerable: bool,
            configurable: bool,
        ) -> Self {
            PropertyDescriptor::Data {
                value,
                writable,
                enumerable,
                configurable,
            }
        }

        pub fn is_data_descriptor(&self) -> bool {
            matches!(self, PropertyDescriptor::Data { .. })
        }

        pub fn is_accessor_descriptor(&self) -> bool {
            matches!(self, PropertyDescriptor::Accessor { .. })
        }

        pub fn is_generic_descriptor(&self) -> bool {
            !matches!(self, PropertyDescriptor::NonGeneric { .. })
        }

        pub fn enumerable(&self) -> bool {
            match self {
                PropertyDescriptor::Data { enumerable, .. } => *enumerable,
                PropertyDescriptor::Accessor { enumerable, .. } => *enumerable,
                PropertyDescriptor::NonGeneric { fields, .. } => {
                    if let Some(Value::Boolean(enumerable)) = fields.get("enumerable") {
                        *enumerable
                    } else {
                        false
                    }
                }
            }
        }

        pub fn configurable(&self) -> bool {
            match self {
                PropertyDescriptor::Data { configurable, .. } => *configurable,
                PropertyDescriptor::Accessor { configurable, .. } => *configurable,
                PropertyDescriptor::NonGeneric { fields, .. } => {
                    if let Some(Value::Boolean(configurable)) = fields.get("configurable") {
                        *configurable
                    } else {
                        false
                    }
                }
            }
        }

        pub fn field(&self, name: &str) -> Option<Value> {
            match self {
                PropertyDescriptor::Data {
                    value,
                    writable,
                    enumerable,
                    configurable,
                } => match name {
                    "value" => Some(value.clone()),
                    "writable" => Some(Value::Boolean(*writable)),
                    "enumerable" => Some(Value::Boolean(*enumerable)),
                    "configurable" => Some(Value::Boolean(*configurable)),
                    _ => None,
                },
                PropertyDescriptor::Accessor {
                    get,
                    set,
                    enumerable,
                    configurable,
                } => match name {
                    "get" => Some(get.clone()),
                    "set" => Some(set.clone()),
                    "enumerable" => Some(Value::Boolean(*enumerable)),
                    "configurable" => Some(Value::Boolean(*configurable)),
                    _ => None,
                },
                PropertyDescriptor::NonGeneric { fields } => fields.get(name).cloned(),
            }
        }

        pub fn set_field(&mut self, name: &str, value: Value) {
            match self {
                PropertyDescriptor::Data {
                    value: val,
                    writable,
                    enumerable,
                    configurable,
                } => match name {
                    "value" => *val = value,
                    "writable" => {
                        if let Value::Boolean(b) = value {
                            *writable = b;
                        }
                    }
                    "enumerable" => {
                        if let Value::Boolean(b) = value {
                            *enumerable = b;
                        }
                    }
                    "configurable" => {
                        if let Value::Boolean(b) = value {
                            *configurable = b;
                        }
                    }
                    _ => {}
                },
                PropertyDescriptor::Accessor {
                    get,
                    set,
                    enumerable,
                    configurable,
                } => match name {
                    "get" => *get = value,
                    "set" => *set = value,
                    "enumerable" => {
                        if let Value::Boolean(b) = value {
                            *enumerable = b;
                        }
                    }
                    "configurable" => {
                        if let Value::Boolean(b) = value {
                            *configurable = b;
                        }
                    }
                    _ => {}
                },
                PropertyDescriptor::NonGeneric { fields } => {
                    fields.insert(name.to_string(), value);
                }
            }
        }
    }

    pub trait ObjectTrait {
        fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>>;
        fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool;

        fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor>;
        fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool;
        fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool;
    }

    #[derive(Debug, Clone)]
    pub struct ArrayObject {
        pub extensible: bool,

        /// NOTE: This is stored as a property under Object, but is stored here for easier access
        /// and it also has slight performance benefits. Plus storing as an object property makes
        /// the length a Number (f64) instead of a u32, which is not ideal.
        pub length: u32,
        pub object: OrdinaryObject,

        pub data: Vec<Value>,
    }

    #[derive(Debug, Clone)]
    pub struct OrdinaryObject {
        pub prototype: Rc<RefCell<Option<Object>>>,
        pub extensible: bool,

        pub properties: HashMap<PropertyKey, PropertyDescriptor>,
    }

    impl OrdinaryObject {
        pub fn prototype() -> OrdinaryObject {
            OrdinaryObject {
                prototype: Rc::new(RefCell::new(None)),
                extensible: true,
                properties: HashMap::new(),
            }
        }
    }

    impl ObjectTrait for OrdinaryObject {
        fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
            self.prototype.clone()
        }

        fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
            let mut object = Object::Ordinary(self.clone());
            let res = ordinary_set_prototype_of(&mut object, prototype);
            if let Object::Ordinary(obj) = object {
                *self = obj;
            }

            res
        }

        fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
            ordinary_get_own_property(&Object::Ordinary(self.clone()), key)
        }

        fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
            self.properties.insert(key.clone(), desc);
            true
        }

        fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
            let mut obj = Object::Ordinary(self.clone());
            let res = ordinary_set(&mut obj, key, value, receiver);
            if let Object::Ordinary(obj) = obj {
                *self = obj;
            }

            *res.unwrap().unwrapped()
        }
    }

    #[derive(Debug, Clone)]
    pub struct MiscObject {
        pub properties: HashMap<PropertyKey, PropertyDescriptor>,
    }

    impl ObjectTrait for MiscObject {
        fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
            if let Some(desc) = self.properties.get(&PropertyKey::from("prototype")) {
                desc.field("value")
                    .map(|v| Rc::new(RefCell::new(v.unwrap_object())))
                    .unwrap_or_else(|| Rc::new(RefCell::new(None)))
            } else {
                Rc::new(RefCell::new(None))
            }
        }

        fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
            if let Some(desc) = self.properties.get(&PropertyKey::from("prototype")) {
                self.properties.insert(
                    PropertyKey::from("prototype"),
                    PropertyDescriptor::Data {
                        value: Value::Object(prototype.unwrap_or_else(|| Object::prototype())),
                        writable: true,
                        enumerable: false,
                        configurable: true,
                    },
                );

                return true;
            }

            return false;
        }

        fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
            self.properties.get(key).cloned()
        }

        fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
            self.properties.insert(key.clone(), desc);
            true
        }

        fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
            if let Some(desc) = self.properties.get(key) {
                self.properties.insert(
                    key.clone(),
                    PropertyDescriptor::Data {
                        value: value.clone(),
                        writable: desc.enumerable(),
                        enumerable: desc.enumerable(),
                        configurable: desc.configurable(),
                    },
                );

                return true;
            }

            return false;
        }
    }

    #[derive(Debug, Clone)]
    pub enum Object {
        Ordinary(OrdinaryObject),
        Array(ArrayObject),
        Misc(MiscObject),
    }

    impl Object {
        pub fn prototype() -> Object {
            Object::Ordinary(OrdinaryObject::prototype())
        }

        pub fn constructor() -> Object {
            Object::Ordinary(OrdinaryObject::prototype())
        }
    }

    impl ObjectTrait for Object {
        fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
            match self {
                Object::Ordinary(obj) => obj.get_prototype_of(),
                Object::Array(arr) => arr.object.get_prototype_of(),
                Object::Misc(misc) => misc.get_prototype_of(),
            }
        }

        fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
            match self {
                Object::Ordinary(obj) => obj.set_prototype_of(prototype),
                Object::Array(arr) => arr.object.set_prototype_of(prototype),
                Object::Misc(misc) => misc.set_prototype_of(prototype),
            }
        }

        fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
            match self {
                Object::Ordinary(obj) => obj.get_own_property(key),
                Object::Array(arr) => {
                    if *key == PropertyKey::from("length") {
                        return Some(PropertyDescriptor::Data {
                            value: Value::Number(Number(arr.length as f64)),
                            writable: true,
                            enumerable: false,
                            configurable: false,
                        });
                    }

                    arr.object.get_own_property(key)
                }
                Object::Misc(misc) => misc.get_own_property(key),
            }
        }

        fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
            match self {
                Object::Ordinary(obj) => obj.define_own_property(key, desc),
                Object::Array(arr) => arr.object.define_own_property(key, desc),
                Object::Misc(misc) => misc.define_own_property(key, desc),
            }
        }

        fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
            match self {
                Object::Ordinary(obj) => obj.set(key, value, receiver),
                Object::Array(arr) => {
                    let res = arr.object.set(key, value, receiver);
                    if *key == PropertyKey::from("length") {
                        if let Value::Number(n) = value {
                            arr.length = n.0 as u32;
                        }
                    }

                    return res;
                }
                Object::Misc(misc) => misc.set(key, value, receiver),
            }
        }
    }

    // impl OrdinaryObject {
    //     fn get_prototype_of(&self) -> CompletionRecord<Option<Object>>;

    //     fn set_prototype_of(&mut self, prototype: Option<Object>) -> CompletionRecord<bool>;

    //     fn is_extensible(&self) -> CompletionRecord<bool>;

    //     fn prevent_extensions(&mut self) -> CompletionRecord<bool>;

    //     fn get_own_property(
    //         &self,
    //         key: PropertyKey,
    //     ) -> CompletionRecord<Option<PropertyDescriptor>>;

    //     fn define_own_property(
    //         &mut self,
    //         key: PropertyKey,
    //         desc: PropertyDescriptor,
    //     ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    //     fn has_property(
    //         &self,
    //         key: PropertyKey,
    //     ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    //     fn get(
    //         &self,
    //         key: PropertyKey,
    //         receiver: Value,
    //     ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError>>;

    //     fn set(
    //         &mut self,
    //         key: PropertyKey,
    //         value: Value,
    //         receiver: Value,
    //     ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    //     fn delete(
    //         &mut self,
    //         key: PropertyKey,
    //     ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    //     fn own_property_keys(&self) -> CompletionRecord<Vec<PropertyKey>>;
    // }

    // impl Ordinary for OrdinaryObject {
    //
    // }
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
}

#[derive(Clone, Debug)]
pub enum ReferenceBase {
    Value(Value),
    EnvironmentRecord(EnvironmentRecord),
    Unresolvable,
}

#[derive(Clone, Debug)]
pub struct Reference {
    pub base: ReferenceBase,
    pub referenced_name: Value,
    pub strict: bool,
    pub this_value: Option<Value>,
}

pub fn same_type(x: &Value, y: &Value) -> bool {
    match (x, y) {
        (Value::Undefined, Value::Undefined)
        | (Value::Null, Value::Null)
        | (Value::Boolean(_), Value::Boolean(_))
        | (Value::String(_), Value::String(_))
        | (Value::Symbol(_), Value::Symbol(_))
        | (Value::Number(_), Value::Number(_))
        | (Value::BigInt(_), Value::BigInt(_))
        | (Value::Object(_), Value::Object(_)) => true,
        _ => false,
    }
}

pub fn same_value(x: &Value, y: &Value) -> bool {
    if !same_type(x, y) {
        return false;
    }

    if let Value::Number(x_num) = x
        && let Value::Number(y_num) = y
    {
        return x_num.same_value(y_num);
    }

    return same_value_non_number(x, y);
}

pub fn same_value_zero(x: &Value, y: &Value) -> bool {
    if !same_type(x, y) {
        return false;
    }

    if let Value::Number(x_num) = x
        && let Value::Number(y_num) = y
    {
        return x_num.same_value_zero(y_num);
    }

    return same_value_non_number(x, y);
}

pub fn same_value_non_number(x: &Value, y: &Value) -> bool {
    if let Value::Undefined = x {
        return true;
    }
    if let Value::Null = x {
        return true;
    }

    if let Value::String(x_str) = x
        && let Value::String(y_str) = y
    {
        return x_str == y_str;
    }

    if let Value::Boolean(x_bool) = x
        && let Value::Boolean(y_bool) = y
    {
        return x_bool == y_bool;
    }

    return false;
}
