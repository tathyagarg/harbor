use std::{cell::RefCell, rc::Rc};

use crate::{
    html5::environments::HostDefined,
    js::{
        executable::{
            context::{
                CodeExecutionContext, ExecutionContext, GenericExecutionContext, ScriptOrModule,
                pop_execution_context, push_execution_context,
            },
            environment::{
                EnvironmentRecord, create_global_function_binding, create_global_var_binding,
            },
            realm::Realm,
        },
        expr::ZigString,
        semantics::{
            evaluate::statements::script_evaluate,
            instantiate_ordinary_function_object,
            r#static::{OwnedParseNode, ParseNode, StaticSemantics},
        },
        stmt::Script,
        types::completion_record::{CompletionRecord, CompletionRecordNormal},
        values::{
            ReferenceOrValue, Value,
            object::{Object, ordinary_function_create},
        },
    },
};

#[derive(Debug, Clone)]
pub struct ScriptRecord {
    pub realm: Rc<RefCell<Realm>>,
    pub ecma_script_code: Script,

    pub host_defined: HostDefined,

    pub loaded_modules: Vec<()>,
}

pub fn parse_script(
    text: &str,
    realm: Rc<RefCell<Realm>>,
    host_defined: &HostDefined,
) -> ScriptRecord {
    println!("text: {}", text);

    let utf16 = text.encode_utf16().collect::<Vec<u16>>();
    let zs = ZigString {
        data: utf16.as_ptr(),
        len: utf16.len(),
    };

    let script = unsafe { crate::js::parse_script(zs) };
    println!("Script: {:?}", script);

    ScriptRecord {
        realm,
        ecma_script_code: script,
        loaded_modules: Vec::new(),
        host_defined: host_defined.clone(),
    }
}

pub fn script_evaluation(script_rec: Rc<ScriptRecord>) -> CompletionRecord<ReferenceOrValue> {
    let global_env = script_rec.realm.borrow().global_env.clone().unwrap();
    let script_context = Rc::new(RefCell::new(ExecutionContext::Code(CodeExecutionContext {
        execution_context: GenericExecutionContext {
            function: None,
            realm: script_rec.realm.clone(),
            script_or_module: Some(ScriptOrModule::Script(script_rec.clone())),
        },

        variable_env: global_env.clone(),
        lexical_env: global_env.clone(),
    })));

    push_execution_context(script_context);

    let script = &script_rec.ecma_script_code;
    global_declaration_instantiation(script, global_env.clone());

    let result = script_evaluate(script);
    let final_res = CompletionRecordNormal(result);

    pop_execution_context();

    final_res
}

pub fn global_declaration_instantiation(script: &Script, env: Rc<RefCell<EnvironmentRecord>>) {
    println!("Global declaration instantiation for script");
    let script_node = ParseNode::Script(script);

    let var_decls = script_node.var_scoped_declarations();
    let mut funcs_to_init = Vec::new();
    let mut declared_func_names = Vec::new();

    for decl in &var_decls {
        if matches!(decl, OwnedParseNode::HoistabeDeclaration(_)) {
            let fn_name = decl.bound_names().first().unwrap().clone();
            println!("Found function declaration: {:?}", fn_name);
            if !declared_func_names.contains(&fn_name) {
                declared_func_names.push(fn_name.clone());
                funcs_to_init.insert(0, decl);
            }
        }
    }

    let mut declared_var_names = Vec::new();
    for decl in &var_decls {
        if matches!(decl, OwnedParseNode::LexicalDeclaration(_)) {
            for name in decl.bound_names() {
                if !declared_func_names.contains(&name) && !declared_var_names.contains(&name) {
                    declared_var_names.push(name.clone());
                }
            }
        }
    }

    let lex_decls = script_node.lexically_scoped_declarations();

    for decl in lex_decls {
        for name in decl.bound_names() {
            println!("Processing lexical declaration: {:?}", name,);
            if decl.is_constant_decl() {
                env.borrow_mut()
                    .create_immutable_binding(name.clone(), true)
                    .unwrap();
            } else {
                env.borrow_mut()
                    .create_mutable_binding(name.clone(), false)
                    .unwrap();
            }
        }
    }

    for func in funcs_to_init {
        if let OwnedParseNode::HoistabeDeclaration(decl) = func {
            let fn_name = func.bound_names().first().unwrap().clone();
            let func_obj = instantiate_ordinary_function_object(decl, env.clone());

            create_global_function_binding(
                env.clone(),
                fn_name,
                &Value::Object(Object::Function(func_obj)),
                false,
            )
            .unwrap();
        }
    }

    for var in declared_var_names {
        create_global_var_binding(env.clone(), var, false).unwrap();
    }
}
