use crate::js::{
    collect_seq,
    semantics::r#static::string_value::string_value,
    stmt::{DECLARATION_LEXICAL_DECLARATION, Declaration, FormalParameter, LexicalDeclaration},
    values::string::JsString,
};

pub enum BoundNames<'a> {
    Declaration(&'a Declaration),
    LexicalDeclaration(&'a LexicalDeclaration),
    FormalParameters(&'a Vec<FormalParameter>),
}

fn bound_names_lexical_declaration(lex_decl: &LexicalDeclaration) -> Vec<JsString> {
    let bindings = collect_seq(&lex_decl.bindings);

    let mut names = Vec::new();

    for binding in bindings {
        let identifier = unsafe { *binding.name };
        let name = string_value(identifier);

        names.push(name);
    }

    names
}

fn bound_names_declaration(declaration: &Declaration) -> Vec<JsString> {
    match declaration.tag {
        DECLARATION_LEXICAL_DECLARATION => {
            let lexical_decl = unsafe { *declaration.data.lex_decl };
            bound_names_lexical_declaration(&lexical_decl)
        }
        _ => vec![],
    }
}

fn bound_names_formal_params(formal_params: &Vec<FormalParameter>) -> Vec<JsString> {
    let mut names = Vec::new();

    for param in formal_params {
        let identifier = unsafe { *param.name };
        let name = string_value(identifier);

        names.push(name);
    }

    names
}

pub fn bound_names(target: BoundNames) -> Vec<JsString> {
    match target {
        BoundNames::Declaration(decl) => bound_names_declaration(decl),
        BoundNames::LexicalDeclaration(lex_decl) => bound_names_lexical_declaration(lex_decl),
        BoundNames::FormalParameters(formal_params) => bound_names_formal_params(formal_params),
    }
}
