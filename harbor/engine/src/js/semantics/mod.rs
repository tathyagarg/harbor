use std::{cell::RefCell, rc::Rc};

use crate::js::{
    behaviours::builtin_functions::FUNCTION_PROTOTYPE,
    collect_seq,
    executable::environment::EnvironmentRecord,
    semantics::r#static::{ParseNode, string_value},
    stmt::HoistableDeclaration,
    values::{
        object::{
            FunctionCreateMode, FunctionObject, PropertyKey, ordinary_function_create,
            set_function_name,
        },
        string::JsString,
    },
};

pub mod evaluate;
pub mod r#static;

pub fn instantiate_ordinary_function_object(
    node: &HoistableDeclaration,
    env: Rc<RefCell<EnvironmentRecord>>,
) -> FunctionObject {
    let name = string_value(unsafe { (*node.name).value.value });
    let source_text = unsafe { *node.body };

    let mut f = ordinary_function_create(
        FUNCTION_PROTOTYPE.clone(),
        JsString::empty(),
        collect_seq(&node.params),
        ParseNode::BlockStatement(&source_text),
        FunctionCreateMode::NonLexicalThis,
        env,
    );

    set_function_name(&mut f, &PropertyKey::String(name));

    f
}
