use std::{cell::RefCell, rc::Rc};

use crate::js::{
    executable::environment::EnvironmentRecord,
    semantics::r#static::string_value,
    stmt::HoistableDeclaration,
    values::object::{FunctionObject, ordinary_function_create},
};

pub mod evaluate;
pub mod r#static;

pub fn instantiate_ordinary_function_object(
    node: &HoistableDeclaration,
    env: Rc<RefCell<EnvironmentRecord>>,
) -> FunctionObject {
    let name = string_value(unsafe { (*node.name).value.value });
    let source_text = unsafe { *node.body };

    // let f = ordinary_function_create()

    todo!()
}
