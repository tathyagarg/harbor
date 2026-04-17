use crate::js::stmt::{DECLARATION_LEXICAL_DECLARATION, Declaration};

pub fn is_constant_decl(declaration: &Declaration) -> bool {
    match declaration.tag {
        DECLARATION_LEXICAL_DECLARATION => {
            let lexical_decl = unsafe { *declaration.data.lex_decl };
            lexical_decl.is_const
        }
        _ => false,
    }
}
