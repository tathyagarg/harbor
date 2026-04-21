use std::{cell::RefCell, rc::Rc, str::FromStr};

use crate::js::{
    behaviours::exotics::arguments::{
        create_mapped_arguments_object, create_unmapped_arguments_object,
    },
    executable::{
        agent::running_execution_context,
        environment::{EnvRecordTrait, new_declarative_environment},
    },
    operations::{create_list_iterator_record, iterator_binding_initialization},
    semantics::r#static::{
        OwnedParseNode, ParseNode, StaticSemantics, contains_expression, is_simple_parameter_list,
    },
    types::completion_record::{CRKThrow, CompletionRecord, CompletionRecordNormal, UNUSED},
    values::{
        Value,
        object::{FunctionObject, Object, ThisMode},
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
    let formals = ParseNode::FormalParameters(&func.formal_parameters);

    let parameter_names = formals.bound_names();
    let has_duplicates = HAS_DUPLICATES!(parameter_names);

    let simple_param_list = is_simple_parameter_list(&func.formal_parameters);
    let has_parameter_expressions = contains_expression(&func.formal_parameters);

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

    {
        let mut env_borrow = env.borrow_mut();
        for param_name in &parameter_names {
            let already_declared = env_borrow.has_binding(&param_name);

            if !already_declared {
                env_borrow.create_mutable_binding(&param_name, false);

                if has_duplicates {
                    env_borrow.initialize_binding(&param_name, &Value::Undefined);
                }
            }
        }
    }

    let param_bindings = if args_object_needed {
        let ao = if strict || !simple_param_list {
            Object::Ordinary(create_unmapped_arguments_object(&args))
        } else {
            Object::Arguments(create_mapped_arguments_object(
                &Object::Function(func.clone()),
                &formals,
                &args,
                env.clone(),
            ))
        };

        if strict {
            env.borrow_mut()
                .create_immutable_binding(&JsString::from_str("arguments").unwrap(), false);
        } else {
            env.borrow_mut()
                .create_mutable_binding(&JsString::from_str("arguments").unwrap(), false);
        }

        env.borrow_mut().initialize_binding(
            &JsString::from_str("arguments").unwrap(),
            &Value::Object(ao),
        );

        [
            vec![JsString::from_str("arguments").unwrap()],
            parameter_names,
        ]
        .concat()
    } else {
        parameter_names
    };

    let iter_rec = create_list_iterator_record(args);
    let used_env = if has_duplicates {
        None
    } else {
        Some(env.clone())
    };

    iterator_binding_initialization(&formals, &mut iter_rec.clone(), used_env.clone()).unwrap();

    let var_env = if !has_parameter_expressions {
        let mut instantiated_var_names = param_bindings.clone();
        for name in &var_names {
            if !instantiated_var_names.contains(name) {
                instantiated_var_names.push(name.clone());

                env.borrow_mut().create_mutable_binding(&name, false);

                env.borrow_mut()
                    .initialize_binding(&name, &Value::Undefined);
            }
        }

        env.clone()
    } else {
        todo!("Handle var declarations when parameter expressions are present")
    };

    let lex_env = if strict {
        var_env.clone()
    } else {
        Rc::new(RefCell::new(new_declarative_environment(Some(var_env))))
    };

    callee_context
        .borrow_mut()
        .replace_lexical_env(lex_env.clone());

    let lex_decls = code_node.lexically_scoped_declarations();
    for decl in lex_decls {
        for name in decl.bound_names() {
            if decl.is_constant_decl() {
                lex_env.borrow_mut().create_immutable_binding(&name, true);
            } else {
                lex_env.borrow_mut().create_mutable_binding(&name, false);
            }
        }
    }

    return Ok(CompletionRecordNormal(()));
}
