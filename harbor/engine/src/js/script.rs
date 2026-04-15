use std::{cell::RefCell, rc::Rc};

use crate::js::{
    executable::{environment::EnvironmentRecord, realm::Realm},
    expr::ZigString,
    stmt::Script,
    syntax::{
        bound_names_declaration, is_constant_decl, lexically_declared_names_script,
        lexically_scoped_declarations, var_declared_names_script,
    },
};

pub struct ScriptRecord {
    pub realm: Rc<RefCell<Realm>>,
    pub ecma_script_code: Script,

    pub loaded_modules: Vec<()>,
}

pub fn parse_script(text: &str, realm: Rc<RefCell<Realm>>) -> ScriptRecord {
    let utf16 = text.encode_utf16().collect::<Vec<u16>>();
    let zs = ZigString {
        data: utf16.as_ptr(),
        len: utf16.len(),
    };

    let script = unsafe { crate::js::parse_text(zs) };
    unsafe { crate::js::free_string(zs) };

    ScriptRecord {
        realm,
        ecma_script_code: script,
        loaded_modules: Vec::new(),
    }
}

pub fn global_declaration_instantiation(script: &Script, env: Rc<RefCell<EnvironmentRecord>>) {
    // let lex_names = lexically_declared_names_script(script);
    // let var_names = var_declared_names_script(script);

    let lex_decls = lexically_scoped_declarations(script);

    for decl in lex_decls {
        for name in bound_names_declaration(&decl) {
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
