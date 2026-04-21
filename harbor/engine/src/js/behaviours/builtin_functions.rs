#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc, sync::LazyLock};

use crate::js::{
    executable::{
        context::{
            ExecutionContext, GenericExecutionContext, pop_execution_context,
            push_execution_context,
        },
        realm::Realm,
    },
    values::{
        Value,
        object::{Object, ObjectTrait, OrdinaryObject, PropertyDescriptor, PropertyKey},
    },
};

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub prototype: Rc<RefCell<Option<Object>>>,
    pub extensible: bool,

    pub realm: Option<Rc<RefCell<Realm>>>,
    pub initial_name: String,
    pub is_async: bool,

    pub internal_closure: fn(this: &Value, args: Vec<Value>) -> Value,
}

pub const FUNCTION_PROTOTYPE: LazyLock<Object> = LazyLock::new(|| {
    Object::BuiltinFunction(BuiltinFunction {
        prototype: Rc::new(RefCell::new(Some(Object::Ordinary(
            OrdinaryObject::prototype(),
        )))),
        extensible: true,
        realm: None,
        initial_name: "Function.prototype".to_string(),
        is_async: false,
        internal_closure: |_this, _args| Value::Undefined,
    })
});

impl ObjectTrait for BuiltinFunction {
    const CALLABLE: bool = true;
    const CONSTRUCTOR: bool = false;

    fn get_prototype_of(&self) -> Rc<RefCell<Option<Object>>> {
        panic!("get_prototype_of not implemented for BuiltinFunction");
    }

    fn set_prototype_of(&mut self, _prototype: Option<Object>) -> bool {
        panic!("set_prototype_of not implemented for BuiltinFunction");
    }

    fn has_property(&self, _key: &PropertyKey) -> bool {
        panic!("has_property not implemented for BuiltinFunction");
    }

    fn get_own_property(&self, _key: &PropertyKey) -> Option<PropertyDescriptor> {
        panic!("get_own_property not implemented for BuiltinFunction");
    }

    fn define_own_property(&mut self, _key: &PropertyKey, _desc: PropertyDescriptor) -> bool {
        panic!("define_own_property not implemented for BuiltinFunction");
    }

    fn get(&self, _key: &PropertyKey, _receiver: &Value) -> Option<Value> {
        panic!("get not implemented for BuiltinFunction");
    }

    fn set(&mut self, _key: &PropertyKey, _value: &Value, _receiver: &mut Value) -> bool {
        panic!("set not implemented for BuiltinFunction");
    }

    fn delete(&mut self, _key: &PropertyKey) -> bool {
        panic!("delete not implemented for BuiltinFunction");
    }

    fn call(&self, this: &Value, args: Vec<Value>) -> Value {
        builtin_call_or_construct(self, this, args)
    }

    fn construct(&self, _args: Vec<Value>, _new_target: &Object) -> Object {
        panic!("construct not implemented for BuiltinFunction");
    }
}

pub fn builtin_call_or_construct(func: &BuiltinFunction, this: &Value, args: Vec<Value>) -> Value {
    let callee_ctx = Rc::new(RefCell::new(ExecutionContext::Generic(
        GenericExecutionContext {
            function: None,
            generator: None,
            realm: func.realm.as_ref().unwrap().clone(),
            script_or_module: None,
        },
    )));

    push_execution_context(callee_ctx);

    let res = (func.internal_closure)(this, args);

    pop_execution_context();

    res
}
