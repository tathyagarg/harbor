use std::{cell::RefCell, rc::Rc, str::FromStr};

use crate::js::{
    executable::{agent::running_execution_context, environment::new_declarative_environment},
    semantics::r#static::{
        OwnedParseNode, ParseNode, StaticSemantics, contains_expression, is_simple_parameter_list,
    },
    types::completion_record::{CRKThrow, CompletionRecord, UNUSED},
    values::{
        Value,
        object::{FunctionObject, ThisMode},
        string::JsString,
    },
};

macro_rules! HAS_DUPLICATES {
    ($names:expr) => {{
        let mut seen = std::collections::HashSet::new();
        !$names.iter().all(|name| seen.insert(name))
    }};
}

pub fn function_declaration_instantiation(
    func: &FunctionObject,
    args: Vec<Value>,
) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED, CRKThrow>> {
    let callee_context = running_execution_context().unwrap();

    let code = func.ecmascript_code;
    let code_node = ParseNode::BlockStatement(&code);

    let strict = func.strict;
    let formals = &func.formal_parameters;

    let parameter_names = ParseNode::FormalParameters(formals).bound_names();
    let has_duplicates = HAS_DUPLICATES!(parameter_names);

    let simple_param_list = is_simple_parameter_list(formals.clone());
    let has_parameter_expressions = contains_expression(formals.clone());

    let var_names = code_node.var_declared_names();
    let var_decls = code_node.var_scoped_declarations();
    let lex_names = code_node.lexically_declared_names();

    let mut func_names = Vec::new();
    let mut funcs_to_init = Vec::new();

    for name in var_decls.iter().rev() {
        if !matches!(name, OwnedParseNode::LexicalDeclaration(_)) {
            assert!(matches!(name, OwnedParseNode::HoistabeDeclaration(_)));

            let fn_name = name.bound_names().first().unwrap().clone();
            if !func_names.contains(&fn_name) {
                func_names.insert(0, fn_name);
                funcs_to_init.insert(0, name);
            }
        }
    }

    let mut args_object_needed = true;

    if func.this_mode == ThisMode::Lexical {
        args_object_needed = false;
    } else if parameter_names.contains(&JsString::from_str("arguments").unwrap()) {
        args_object_needed = false;
    } else if !has_parameter_expressions {
        if func_names.contains(&JsString::from_str("arguments").unwrap())
            || lex_names.contains(&JsString::from_str("arguments").unwrap())
        {
            args_object_needed = false;
        }
    }

    let env = if strict || !has_parameter_expressions {
        callee_context.borrow().lexical_env().unwrap()
    } else {
        let callee_env = callee_context.borrow().lexical_env().unwrap();
        let new_env = Rc::new(RefCell::new(new_declarative_environment(Some(callee_env))));

        callee_context
            .borrow_mut()
            .replace_lexical_env(new_env.clone());

        new_env
    };

    let mut env_borrow = env.borrow_mut();
    for param_name in parameter_names {
        let already_declared = env_borrow.has_binding(&param_name).unwrap().value;

        if !already_declared {
            env_borrow
                .create_mutable_binding(param_name.clone(), false)
                .unwrap();

            if has_duplicates {
                env_borrow
                    .initialize_binding(param_name.clone(), &Value::Undefined)
                    .unwrap();
            }
        }
    }

    if args_object_needed {
        let ao = if strict || !simple_param_list {
        } else {
        };
    }

    todo!()
}
