mod bound_names;
mod lexically_declared_names;
mod lexically_scoped_declarations;
mod string_value;
mod top_level_var_scoped_decls;
mod var_declared_names;
mod var_scoped_declarations;

use std::fmt::Debug;

// pub use bound_names::bound_names;
// pub use lexically_declared_names::lexically_declared_names;
// pub use lexically_scoped_declarations::lexically_scoped_declarations;
pub use string_value::string_value;
// pub use var_declared_names::var_declared_names;
// pub use var_scoped_declarations::var_scoped_declarations;

use crate::js::{
    stmt::{
        BlockStatement, Declaration, FormalParameter, HoistableDeclaration, IfStatement,
        LexicalBinding, LexicalDeclaration, Script, SeqStatementOrDeclaration, Statement,
        StatementOrDeclaration, WhileStatement,
    },
    values::string::JsString,
};

pub trait StaticSemantics {
    fn bound_names(&self) -> Vec<JsString>;
    fn lexically_declared_names(&self) -> Vec<JsString>;
    fn lexically_scoped_declarations(&self) -> Vec<OwnedParseNode>;
    fn var_declared_names(&self) -> Vec<JsString>;
    fn var_scoped_declarations(&self) -> Vec<OwnedParseNode>;
    fn top_level_var_scoped_decls(&self) -> Vec<OwnedParseNode>;

    fn is_constant_decl(&self) -> bool;
}

pub enum ParseNode<'a> {
    Declaration(&'a Declaration),
    LexicalDeclaration(&'a LexicalDeclaration),
    FormalParameters(&'a Vec<FormalParameter>),
    HoistabeDeclaration(&'a HoistableDeclaration),

    Script(&'a Script),
    Statement(&'a Statement),
    StatementOrDeclList(&'a SeqStatementOrDeclaration),
    StatmentOrDeclaration(&'a StatementOrDeclaration),
    BlockStatement(&'a BlockStatement),
    IfStatement(&'a IfStatement),
    WhileStatement(&'a WhileStatement),

    LexicalBinding(&'a LexicalBinding),
}

impl<'a> StaticSemantics for ParseNode<'a> {
    fn bound_names(&self) -> Vec<JsString> {
        bound_names::bound_names(self)
    }

    fn lexically_declared_names(&self) -> Vec<JsString> {
        lexically_declared_names::lexically_declared_names(self)
    }

    fn lexically_scoped_declarations(&self) -> Vec<OwnedParseNode> {
        lexically_scoped_declarations::lexically_scoped_declarations(self)
    }

    fn var_declared_names(&self) -> Vec<JsString> {
        var_declared_names::var_declared_names(self)
    }

    fn var_scoped_declarations(&self) -> Vec<OwnedParseNode> {
        var_scoped_declarations::var_scoped_declarations(self)
    }

    fn top_level_var_scoped_decls(&self) -> Vec<OwnedParseNode> {
        top_level_var_scoped_decls::top_level_var_scoped_decls(self)
    }

    fn is_constant_decl(&self) -> bool {
        match self {
            ParseNode::LexicalDeclaration(decl) => decl.is_const,
            ParseNode::HoistabeDeclaration(_) => false,
            _ => panic!("is_constant_decl is only applicable to LexicalDeclaration nodes"),
        }
    }
}

pub enum OwnedParseNode {
    Declaration(Declaration),
    LexicalDeclaration(LexicalDeclaration),
    FormalParameters(Vec<FormalParameter>),
    HoistabeDeclaration(HoistableDeclaration),

    Script(Script),
    Statement(Statement),
    StatementOrDeclList(SeqStatementOrDeclaration),
    StatmentOrDeclaration(StatementOrDeclaration),
    BlockStatement(BlockStatement),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),

    LexicalBinding(LexicalBinding),
}

impl StaticSemantics for OwnedParseNode {
    fn bound_names(&self) -> Vec<JsString> {
        bound_names::bound_names_owned(self)
    }

    fn lexically_declared_names(&self) -> Vec<JsString> {
        lexically_declared_names::lexically_declared_names_owned(self)
    }

    fn lexically_scoped_declarations(&self) -> Vec<OwnedParseNode> {
        lexically_scoped_declarations::lexically_scoped_declarations_owned(self)
    }

    fn var_declared_names(&self) -> Vec<JsString> {
        var_declared_names::var_declared_names_owned(self)
    }

    fn var_scoped_declarations(&self) -> Vec<OwnedParseNode> {
        var_scoped_declarations::var_scoped_declarations_owned(self)
    }

    fn top_level_var_scoped_decls(&self) -> Vec<OwnedParseNode> {
        top_level_var_scoped_decls::top_level_var_scoped_decls_owned(self)
    }

    fn is_constant_decl(&self) -> bool {
        match self {
            OwnedParseNode::LexicalDeclaration(decl) => decl.is_const,
            OwnedParseNode::HoistabeDeclaration(_) => false,
            _ => panic!(
                "is_constant_decl is only applicable to LexicalDeclaration nodes, got {:?}",
                self
            ),
        }
    }
}

impl<'a> Debug for ParseNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseNode::Declaration(_) => write!(f, "ParseNode::Declaration"),
            ParseNode::LexicalDeclaration(_) => write!(f, "ParseNode::LexicalDeclaration"),
            ParseNode::FormalParameters(_) => write!(f, "ParseNode::FormalParameters"),
            ParseNode::HoistabeDeclaration(_) => write!(f, "ParseNode::HoistabeDeclaration"),
            ParseNode::Script(_) => write!(f, "ParseNode::Script"),
            ParseNode::Statement(_) => write!(f, "ParseNode::Statement"),
            ParseNode::StatementOrDeclList(_) => write!(f, "ParseNode::StatementOrDeclList"),
            ParseNode::StatmentOrDeclaration(_) => write!(f, "ParseNode::StatmentOrDeclaration"),
            ParseNode::BlockStatement(_) => write!(f, "ParseNode::BlockStatement"),
            ParseNode::IfStatement(_) => write!(f, "ParseNode::IfStatement"),
            ParseNode::WhileStatement(_) => write!(f, "ParseNode::WhileStatement"),
            ParseNode::LexicalBinding(_) => write!(f, "ParseNode::LexicalBinding"),
        }
    }
}

impl Debug for OwnedParseNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnedParseNode::Declaration(_) => write!(f, "OwnedParseNode::Declaration"),
            OwnedParseNode::LexicalDeclaration(_) => {
                write!(f, "OwnedParseNode::LexicalDeclaration")
            }
            OwnedParseNode::FormalParameters(_) => write!(f, "OwnedParseNode::FormalParameters"),
            OwnedParseNode::HoistabeDeclaration(_) => {
                write!(f, "OwnedParseNode::HoistabeDeclaration")
            }
            OwnedParseNode::Script(_) => write!(f, "OwnedParseNode::Script"),
            OwnedParseNode::Statement(_) => write!(f, "OwnedParseNode::Statement"),
            OwnedParseNode::StatementOrDeclList(_) => {
                write!(f, "OwnedParseNode::StatementOrDeclList")
            }
            OwnedParseNode::StatmentOrDeclaration(_) => {
                write!(f, "OwnedParseNode::StatmentOrDeclaration")
            }
            OwnedParseNode::BlockStatement(_) => write!(f, "OwnedParseNode::BlockStatement"),
            OwnedParseNode::IfStatement(_) => write!(f, "OwnedParseNode::IfStatement"),
            OwnedParseNode::WhileStatement(_) => write!(f, "OwnedParseNode::WhileStatement"),
            OwnedParseNode::LexicalBinding(_) => write!(f, "OwnedParseNode::LexicalBinding"),
        }
    }
}

pub fn is_simple_parameter_list(formals: &Vec<FormalParameter>) -> bool {
    if formals.is_empty() {
        return true;
    }

    for param in formals {
        if param.is_rest || (unsafe { *param.initializer }).has_value {
            return false;
        }
    }

    true
}

pub fn contains_expression(formals: &Vec<FormalParameter>) -> bool {
    for param in formals {
        if (unsafe { *param.initializer }).has_value {
            return true;
        }
    }

    false
}
