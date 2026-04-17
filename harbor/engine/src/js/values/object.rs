use std::{cell::RefCell, collections::HashMap, fmt::Debug, ops::Deref, rc::Rc, str::FromStr};

use crate::js::{
    SLOT_PROTOTYPE,
    behaviours::{
        ordinary_get, ordinary_get_own_property, ordinary_set, ordinary_set_prototype_of,
    },
    executable::{
        agent::SURROUNDING_AGENT,
        context::{
            CodeExecutionContext, ExecutionContext, GenericExecutionContext, ScriptOrModule,
        },
        environment::{
            EnvironmentRecord, EnvironmentRecordKind, bind_this_value, new_function_environment,
        },
        realm::Realm,
    },
    operations::to_object,
    stmt::{BlockStatement, FormalParameter},
    types::completion_record::{CRKReturn, CRKThrow, CompletionRecord, UNUSED},
    values::{Value, number::Number, reference::ReferenceName, string::JsString, symbol::Symbol},
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

impl From<ReferenceName> for PropertyKey {
    fn from(value: ReferenceName) -> Self {
        match value {
            ReferenceName::Value(v) => match v {
                Value::String(s) => PropertyKey::String(s),
                Value::Symbol(sym) => PropertyKey::Symbol(sym),
                _ => panic!("Invalid ReferenceName value for PropertyKey"),
            },
            ReferenceName::Private(_) => {
                panic!("Cannot convert Private ReferenceName to PropertyKey")
            }
        }
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
    const CALLABLE: bool;
    const CONSTRUCTOR: bool;

    fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>>;
    fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool;

    fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor>;
    fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool;

    fn get(&self, key: &PropertyKey, receiver: &Value) -> Option<Value>;
    fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool;

    fn call(&self, this: &Value, args: Vec<Value>) -> Value;
    fn construct(&self, args: Vec<Value>, new_target: &Object) -> Object;
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
    /// NOTE: This can be treated as %Object.prototype%
    /// https://tc39.es/ecma262/#sec-properties-of-the-object-prototype-object
    pub fn prototype() -> OrdinaryObject {
        OrdinaryObject {
            prototype: Rc::new(RefCell::new(None)),
            extensible: true,
            properties: HashMap::new(),
        }
    }
}

impl ObjectTrait for OrdinaryObject {
    const CALLABLE: bool = false;
    const CONSTRUCTOR: bool = false;

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

    fn get(&self, key: &PropertyKey, receiver: &Value) -> Option<Value> {
        let obj = Object::Ordinary(self.clone());
        let res = ordinary_get(&obj, key, receiver);

        if let Ok(val) = res {
            Some(val.unwrapped().clone())
        } else {
            None
        }
    }

    fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
        let mut obj = Object::Ordinary(self.clone());
        let res = ordinary_set(&mut obj, key, value, receiver);
        if let Object::Ordinary(obj) = obj {
            *self = obj;
        }

        *res.unwrap().unwrapped()
    }

    fn call(&self, _this: &Value, _args: Vec<Value>) -> Value {
        panic!("Object is not callable")
    }

    fn construct(&self, _args: Vec<Value>, _new_target: &Object) -> Object {
        panic!("Object is not a constructor")
    }
}

#[derive(Debug, Clone)]
pub enum SlotValue {
    Undefined,
    List(Vec<Box<SlotValue>>),
    Value(Value),
}

#[derive(Clone)]
pub struct EssentialMethodProxy<T> {
    pub get_prototype_of: Rc<dyn Fn(&T) -> Rc<RefCell<Option<Object>>>>,
    pub set_prototype_of: Rc<dyn Fn(&mut T, Option<Object>) -> bool>,

    pub get_own_property: Rc<dyn Fn(&T, &PropertyKey) -> Option<PropertyDescriptor>>,
    pub define_own_property: Rc<dyn Fn(&mut T, &PropertyKey, PropertyDescriptor) -> bool>,

    pub get: Rc<dyn Fn(&T, &PropertyKey, &Value) -> Option<Value>>,
    pub set: Rc<dyn Fn(&mut T, &PropertyKey, &Value, &mut Value) -> bool>,
}

impl<T> Debug for EssentialMethodProxy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EssentialMethodProxy").finish()
    }
}

#[derive(Debug, Clone)]
pub struct MiscObject {
    pub internal_slots: HashMap<String, SlotValue>,
    pub properties: HashMap<PropertyKey, PropertyDescriptor>,

    pub method_proxy: Option<EssentialMethodProxy<MiscObject>>,
}

impl MiscObject {
    const _GET_PROTOTYPE_OF: fn(&MiscObject) -> Rc<RefCell<Option<Object>>> = pmdd_get_prototype_of;
    const _SET_PROTOTYPE_OF: fn(&mut MiscObject, Option<Object>) -> bool = pmdd_set_prototype_of;

    const _GET_OWN_PROPERTY: fn(&MiscObject, &PropertyKey) -> Option<PropertyDescriptor> =
        pmdd_get_own_property;
    const _DEFINE_OWN_PROPERTY: fn(&mut MiscObject, &PropertyKey, PropertyDescriptor) -> bool =
        pmdd_define_own_property;

    const _GET: fn(&MiscObject, &PropertyKey, &Value) -> Option<Value> = pmdd_get;
    const _SET: fn(&mut MiscObject, &PropertyKey, &Value, &mut Value) -> bool = pmdd_set;
}

impl ObjectTrait for MiscObject {
    const CALLABLE: bool = false;
    const CONSTRUCTOR: bool = false;

    fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
        if let Some(proxy) = &self.method_proxy {
            return (proxy.get_prototype_of)(self);
        }

        Self::_GET_PROTOTYPE_OF(self)
    }

    fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
        if let Some(proxy) = &self.method_proxy {
            return (proxy.set_prototype_of.clone())(self, prototype);
        }

        Self::_SET_PROTOTYPE_OF(self, prototype)
    }

    fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        if let Some(proxy) = &self.method_proxy {
            return (proxy.get_own_property)(self, key);
        }

        Self::_GET_OWN_PROPERTY(self, key)
    }

    fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
        if let Some(proxy) = &self.method_proxy {
            return (proxy.define_own_property.clone())(self, key, desc);
        }

        Self::_DEFINE_OWN_PROPERTY(self, key, desc)
    }

    fn get(&self, key: &PropertyKey, receiver: &Value) -> Option<Value> {
        if let Some(proxy) = &self.method_proxy {
            return (proxy.get.clone())(self, key, receiver);
        }

        Self::_GET(self, key, receiver)
    }

    fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
        if let Some(proxy) = &self.method_proxy {
            return (proxy.set.clone())(self, key, value, receiver);
        }

        Self::_SET(self, key, value, receiver)
    }

    fn call(&self, _this: &Value, _args: Vec<Value>) -> Value {
        panic!("MiscObject is not callable")
    }

    fn construct(&self, _args: Vec<Value>, _new_target: &Object) -> Object {
        panic!("MiscObject is not a constructor")
    }
}

// NOTE: PMDD stands for "Proxy Method Default Definition".
pub fn pmdd_get_prototype_of(obj: &MiscObject) -> Rc<RefCell<Option<Object>>> {
    if let Some(slot) = obj.internal_slots.get(SLOT_PROTOTYPE) {
        if let SlotValue::Value(Value::Object(obj)) = slot {
            return Rc::new(RefCell::new(Some(obj.clone())));
        }
    }

    if let Some(desc) = obj.properties.get(&PropertyKey::from(SLOT_PROTOTYPE)) {
        desc.field("value")
            .map(|v| Rc::new(RefCell::new(v.unwrap_object())))
            .unwrap_or_else(|| Rc::new(RefCell::new(None)))
    } else {
        Rc::new(RefCell::new(None))
    }
}

fn pmdd_set_prototype_of(obj: &mut MiscObject, prototype: Option<Object>) -> bool {
    if let Some(slot) = obj.internal_slots.get_mut(SLOT_PROTOTYPE) {
        *slot = match prototype {
            Some(obj) => SlotValue::Value(Value::Object(obj)),
            None => SlotValue::Undefined,
        };

        return true;
    }

    if let Some(_) = obj.properties.get(&PropertyKey::from(SLOT_PROTOTYPE)) {
        obj.properties.insert(
            PropertyKey::from(SLOT_PROTOTYPE),
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

fn pmdd_get_own_property(obj: &MiscObject, key: &PropertyKey) -> Option<PropertyDescriptor> {
    obj.properties.get(key).cloned()
}

fn pmdd_define_own_property(
    obj: &mut MiscObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
) -> bool {
    obj.properties.insert(key.clone(), desc);
    true
}

fn pmdd_get(obj: &MiscObject, key: &PropertyKey, _receiver: &Value) -> Option<Value> {
    if let Some(desc) = obj.properties.get(key) {
        if let PropertyDescriptor::Data { value, .. } = desc {
            return Some(value.clone());
        }
    }

    None
}

fn pmdd_set(obj: &mut MiscObject, key: &PropertyKey, value: &Value, _receiver: &mut Value) -> bool {
    if let Some(desc) = obj.properties.get(key) {
        obj.properties.insert(
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

#[derive(Debug, Clone)]
pub enum ConstructorKind {
    Base,
    Derived,
}

#[derive(Debug, Clone, Copy)]
pub enum ThisMode {
    Lexical,
    Strict,
    Global,
}

#[derive(Clone)]
pub struct FunctionObject {
    pub object: OrdinaryObject,

    pub call: Rc<dyn Fn(&Value, Vec<Value>) -> Value>,
    pub construct: Option<Rc<dyn Fn(Vec<Value>, &Object) -> Object>>,

    pub extensible: bool,
    pub prototype: Option<Rc<Object>>,

    pub environment: EnvironmentRecord,
    pub private_env: (), // TODO: Implement private environment

    pub formal_parameters: Vec<FormalParameter>,
    pub ecmascript_code: BlockStatement,

    pub constructor_kind: ConstructorKind,
    pub realm: Rc<RefCell<Realm>>,

    pub script_or_module: ScriptOrModule,
    pub this_mode: ThisMode,

    pub strict: bool,
    pub home_object: Rc<Object>,

    pub source_text: JsString,

    pub fields: (),
    pub private_methods: (),
    pub class_field_initializer_name: (),
    pub is_class_constructor: (),
}

impl Debug for FunctionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionObject")
            .field("object", &self.object)
            .field("extensible", &self.extensible)
            .field("prototype", &self.prototype)
            .field("environment", &self.environment)
            .field("constructor_kind", &self.constructor_kind)
            .field("realm", &self.realm)
            .field("script_or_module", &self.script_or_module)
            .field("this_mode", &self.this_mode)
            .field("strict", &self.strict)
            .field("home_object", &self.home_object)
            .field("source_text", &self.source_text)
            .finish()
    }
}

pub fn prepare_for_ordinary_call(
    func: &FunctionObject,
    new_target: Option<Object>,
) -> Rc<RefCell<ExecutionContext>> {
    let local_env = Rc::new(RefCell::new(new_function_environment(func, new_target)));

    let callee_context = CodeExecutionContext {
        execution_context: GenericExecutionContext {
            function: Some(func.clone()),
            realm: func.realm.clone(),
            script_or_module: Some(func.script_or_module.clone()),
        },
        lexical_env: local_env.clone(),
        variable_env: local_env,
    };

    let ec = Rc::new(RefCell::new(ExecutionContext::Code(callee_context)));

    SURROUNDING_AGENT.with(|agent| {
        if let Some(agent) = agent.borrow().as_ref() {
            agent.borrow_mut().execution_context_stack.push(ec.clone())
        } else {
            panic!("No surrounding agent found");
        }
    });

    ec
}

pub fn ordinary_call_bind_this(
    func: &FunctionObject,
    callee_context: Rc<RefCell<ExecutionContext>>,
    this_arg: &Value,
) -> UNUSED {
    let this_mode = func.this_mode;
    if let ThisMode::Lexical = this_mode {
        return;
    }

    let callee_realm = func.realm.clone();
    let local_env = callee_context.borrow().lexical_env().unwrap();

    let this_value = if let ThisMode::Strict = this_mode {
        this_arg.clone()
    } else {
        if matches!(this_arg, Value::Undefined | Value::Null) {
            let global_env = callee_realm.borrow().global_env.clone();
            if let EnvironmentRecordKind::Global {
                global_this_value, ..
            } = &global_env.unwrap().borrow().kind
            {
                Value::Object(global_this_value.clone().borrow().deref().clone())
            } else {
                panic!("Global environment record does not have a global this value");
            }
        } else {
            Value::Object(to_object(this_arg).unwrap().value)
        }
    };

    bind_this_value(local_env, &this_value).unwrap();
}

pub fn ordinary_call_evaluate_body(
    func: &FunctionObject,
    args: Vec<Value>,
) -> Result<CompletionRecord<Value, CRKReturn>, CompletionRecord<(), CRKThrow>> {
    todo!()
}

impl ObjectTrait for FunctionObject {
    const CALLABLE: bool = true;
    const CONSTRUCTOR: bool = true;

    fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
        self.object.get_prototype_of()
    }

    fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
        self.object.set_prototype_of(prototype)
    }

    fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        self.object.get_own_property(key)
    }

    fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
        self.object.define_own_property(key, desc)
    }

    fn get(&self, key: &PropertyKey, receiver: &Value) -> Option<Value> {
        self.object.get(key, receiver)
    }

    fn set(&mut self, key: &PropertyKey, value: &Value, receiver: &mut Value) -> bool {
        self.object.set(key, value, receiver)
    }

    fn call(&self, this: &Value, args: Vec<Value>) -> Value {
        let callee_context = prepare_for_ordinary_call(self, None);

        ordinary_call_bind_this(self, callee_context.clone(), this);

        let result = ordinary_call_evaluate_body(self, args);

        SURROUNDING_AGENT.with(|agent| {
            if let Some(agent) = agent.borrow().as_ref() {
                agent.borrow_mut().execution_context_stack.pop();
            } else {
                panic!("No surrounding agent found");
            }
        });

        if let Ok(completion) = result {
            return completion.value;
        }

        panic!("Function call threw an exception");
    }

    fn construct(&self, _args: Vec<Value>, _new_target: &Object) -> Object {
        todo!("bleh")
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Ordinary(OrdinaryObject),
    Array(ArrayObject),
    Function(FunctionObject),
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
    const CALLABLE: bool = true;
    const CONSTRUCTOR: bool = true;

    fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
        match self {
            Object::Ordinary(obj) => obj.get_prototype_of(),
            Object::Array(arr) => arr.object.get_prototype_of(),
            Object::Misc(misc) => misc.get_prototype_of(),
            Object::Function(func) => func.get_prototype_of(),
        }
    }

    fn set_prototype_of(&mut self, prototype: Option<Object>) -> bool {
        match self {
            Object::Ordinary(obj) => obj.set_prototype_of(prototype),
            Object::Array(arr) => arr.object.set_prototype_of(prototype),
            Object::Misc(misc) => misc.set_prototype_of(prototype),
            Object::Function(func) => func.set_prototype_of(prototype),
        }
    }

    fn get(&self, key: &PropertyKey, receiver: &Value) -> Option<Value> {
        match self {
            Object::Ordinary(obj) => obj.get(key, receiver),
            Object::Array(arr) => arr.object.get(key, receiver),
            Object::Misc(misc) => misc.get(key, receiver),
            Object::Function(func) => func.get(key, receiver),
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
            Object::Function(func) => func.get_own_property(key),
        }
    }

    fn define_own_property(&mut self, key: &PropertyKey, desc: PropertyDescriptor) -> bool {
        match self {
            Object::Ordinary(obj) => obj.define_own_property(key, desc),
            Object::Array(arr) => arr.object.define_own_property(key, desc),
            Object::Misc(misc) => misc.define_own_property(key, desc),
            Object::Function(func) => func.define_own_property(key, desc),
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
            Object::Function(func) => func.set(key, value, receiver),
        }
    }

    fn call(&self, this: &Value, args: Vec<Value>) -> Value {
        match self {
            Object::Ordinary(obj) => obj.call(this, args),
            Object::Array(arr) => arr.object.call(this, args),
            Object::Misc(misc) => misc.call(this, args),
            Object::Function(func) => func.call(this, args),
        }
    }

    fn construct(&self, args: Vec<Value>, new_target: &Object) -> Object {
        match self {
            Object::Ordinary(obj) => obj.construct(args, new_target),
            Object::Array(arr) => arr.object.construct(args, new_target),
            Object::Misc(misc) => misc.construct(args, new_target),
            Object::Function(func) => func.construct(args, new_target),
        }
    }
}
