use crate::js::{
    executable::agent::running_execution_context,
    semantics::r#static::{BoundNames, bound_names, contains_expression, is_simple_parameter_list},
    types::completion_record::{CRKThrow, CompletionRecord, UNUSED},
    values::{Value, object::FunctionObject},
};

macro_rules! HAS_DUPLICATES {
    ($names:expr) => {{
        let mut seen = std::collections::HashSet::new();
        !$names.iter().all(|name| seen.insert(name))
    }};
}

pub fn function_declaration_instantiation(
    func: &FunctionObject,
    args: Vec<Value>,
) -> Result<CompletionRecord<UNUSED>, CompletionRecord<UNUSED, CRKThrow>> {
    let callee_context = running_execution_context();

    let code = func.ecmascript_code;
    let strict = func.strict;
    let formals = &func.formal_parameters;

    let parameter_names = bound_names(BoundNames::FormalParameters(formals));
    let has_duplicates = HAS_DUPLICATES!(parameter_names);

    let simple_param_list = is_simple_parameter_list(formals.clone());
    let has_parameter_expressions = contains_expression(formals.clone());

    todo!()
}
