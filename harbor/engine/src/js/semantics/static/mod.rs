mod bound_names;
mod string_value;

pub use bound_names::{BoundNames, bound_names};
pub use string_value::string_value;

use crate::js::stmt::FormalParameter;

pub fn is_simple_parameter_list(formals: Vec<FormalParameter>) -> bool {
    todo!()
}
