mod bound_names;
mod string_value;
mod var_declared_names;
mod var_scoped_declarations;

use std::fmt::Debug;

pub use bound_names::bound_names;
pub use string_value::string_value;
pub use var_declared_names::var_declared_names;
pub use var_scoped_declarations::var_scoped_declarations;

use crate::js::stmt::{
    BlockStatement, Declaration, FormalParameter, HoistableDeclaration, IfStatement,
    LexicalDeclaration, Script, Statement, StatementOrDeclaration, WhileStatement,
};

pub enum ParseNode<'a> {
    Declaration(&'a Declaration),
    LexicalDeclaration(&'a LexicalDeclaration),
    FormalParameters(&'a Vec<FormalParameter>),
    HoistabeDeclaration(&'a HoistableDeclaration),

    Script(&'a Script),
    Statement(&'a Statement),
    StatmentOrDeclaration(&'a StatementOrDeclaration),
    BlockStatement(&'a BlockStatement),
    IfStatement(&'a IfStatement),
    WhileStatement(&'a WhileStatement),
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
            ParseNode::StatmentOrDeclaration(_) => write!(f, "ParseNode::StatmentOrDeclaration"),
            ParseNode::BlockStatement(_) => write!(f, "ParseNode::BlockStatement"),
            ParseNode::IfStatement(_) => write!(f, "ParseNode::IfStatement"),
            ParseNode::WhileStatement(_) => write!(f, "ParseNode::WhileStatement"),
        }
    }
}

pub fn is_simple_parameter_list(formals: Vec<FormalParameter>) -> bool {
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

pub fn contains_expression(formals: Vec<FormalParameter>) -> bool {
    for param in formals {
        if (unsafe { *param.initializer }).has_value {
            return true;
        }
    }

    false
}
