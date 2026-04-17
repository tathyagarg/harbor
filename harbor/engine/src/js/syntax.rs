use crate::js::{
    collect_seq,
    stmt::{
        DECLARATION_LEXICAL_DECLARATION, Declaration, STATEMENT_OR_DECLARATION_DECLARATION,
        STATEMENT_OR_DECLARATION_STATEMENT, Script,
    },
};

pub fn is_constant_decl(declaration: &Declaration) -> bool {
    match declaration.tag {
        DECLARATION_LEXICAL_DECLARATION => {
            let lexical_decl = unsafe { *declaration.data.lex_decl };
            lexical_decl.is_const
        }
        _ => false,
    }
}
