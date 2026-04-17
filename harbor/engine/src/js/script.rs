use std::{cell::RefCell, rc::Rc};

use crate::{
    html5::environments::HostDefined,
    js::{
        executable::{
            context::{
                CodeExecutionContext, ExecutionContext, GenericExecutionContext, ScriptOrModule,
                pop_execution_context, push_execution_context,
            },
            environment::EnvironmentRecord,
            realm::Realm,
        },
        expr::ZigString,
        semantics::{
            evaluate::statements::script_evaluate,
            r#static::{ParseNode, bound_names},
        },
        stmt::Script,
        syntax::{is_constant_decl, lexically_scoped_declarations_script},
        types::completion_record::{CompletionRecord, CompletionRecordNormal},
        values::ReferenceOrValue,
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
    unsafe { crate::js::free_string(zs) };

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
    // let lex_names = lexically_declared_names_script(script);
    // let var_names = var_declared_names_script(script);

    let lex_decls = lexically_scoped_declarations_script(script);

    for decl in lex_decls {
        for name in bound_names(ParseNode::Declaration(&decl)) {
            if is_constant_decl(&decl) {
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
}
