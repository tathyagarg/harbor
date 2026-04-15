use crate::js::{
    collect_seq,
    executable::context::resolve_binding,
    semantics::expressions::identifier::string_value,
    stmt::{LexicalBinding, LexicalDeclaration},
    values::{Value, reference::initialize_referenced_binding},
};

pub fn evaluate(declaration: &LexicalDeclaration) {
    let binding_list = collect_seq(&declaration.bindings);
    evaluate_binding_list(&binding_list);
}

fn evaluate_binding_list(bindings: &[LexicalBinding]) {
    for binding in bindings {
        evaluate_binding(binding);
    }
}

fn evaluate_binding(binding: &LexicalBinding) {
    let initializer_wrapper = unsafe { *binding.initializer };

    let identifier = unsafe { *binding.name };
    let binding_id = string_value(identifier);

    let mut lhs = resolve_binding(binding_id, None).unwrap().value;

    if initializer_wrapper.has_value {
        let initializer = unsafe { initializer_wrapper.value.value };

        let rhs = super::super::expressions::assignment::evaluate(&initializer);
        let value = rhs.get_value().unwrap().value;

        initialize_referenced_binding(&mut lhs, &value).unwrap();
    } else {
        initialize_referenced_binding(&mut lhs, &Value::Undefined).unwrap();
    }
}
