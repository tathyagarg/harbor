mod bound_names;
mod string_value;
mod var_declared_names;
mod var_scoped_declarations;

pub use bound_names::{BoundNames, bound_names};
pub use string_value::string_value;
pub use var_declared_names::{VarDeclaredNames, var_declared_names};
pub use var_scoped_declarations::{VarScopedDeclarations, var_scoped_declarations};

use crate::js::stmt::FormalParameter;

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
