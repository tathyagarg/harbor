use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
    sync::LazyLock,
};

use crate::js::{
    behaviours::ordinary_object_create,
    executable::{
        agent::running_execution_context,
        context::{
            CodeExecutionContext, ExecutionContext, GenericExecutionContext, ScriptOrModule,
            pop_execution_context, push_execution_context,
        },
        realm::current_realm,
    },
    operations::create_iterator_result_object,
    values::{
        Value,
        object::{Object, OrdinaryObject, OrdinaryWrapper},
    },
};

pub const ITEREATOR_PROTOTYPE: LazyLock<Object> = LazyLock::new(|| {
    let ordinary =
        ordinary_object_create(Some(Object::Ordinary(OrdinaryObject::prototype())), vec![]);

    Object::Ordinary(ordinary)
});

pub const GENERATOR_PROTOTYPE: LazyLock<Object> = LazyLock::new(|| {
    let iterator_prototype = &*ITEREATOR_PROTOTYPE;
    let ordinary = ordinary_object_create(Some(iterator_prototype.clone()), vec![]);

    Object::Ordinary(ordinary)
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeneratorState {
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

#[derive(Clone, Debug)]
pub struct Generator {
    parent: OrdinaryObject,

    generator_state: GeneratorState,
    generator_context: Weak<RefCell<ExecutionContext>>,
    generator_brand: Option<String>,
}

impl OrdinaryWrapper for Generator {
    fn ordinary(&self) -> &OrdinaryObject {
        &self.parent
    }

    fn ordinary_mut(&mut self) -> &mut OrdinaryObject {
        &mut self.parent
    }
}

pub fn generator_start(generator: &mut Generator, genbody: Box<dyn Fn() -> Option<Value>>) {
    let ctx = running_execution_context().unwrap();

    match ctx.borrow_mut().deref_mut() {
        ExecutionContext::Generic(GenericExecutionContext {
            generator: generator_ref,
            ..
        }) => {
            *generator_ref = Some(generator.clone());
        }
        ExecutionContext::Code(CodeExecutionContext {
            execution_context, ..
        }) => {
            execution_context.generator = Some(generator.clone());
        }
    }

    let closure = || {
        let ac_gen_ctx = running_execution_context().unwrap();

        let result = genbody();
        pop_execution_context();

        match *ac_gen_ctx.borrow_mut() {
            ExecutionContext::Generic(GenericExecutionContext {
                ref mut generator, ..
            }) => {
                generator.as_mut().unwrap().generator_state = GeneratorState::Completed;
            }
            ExecutionContext::Code(CodeExecutionContext {
                ref mut execution_context,
                ..
            }) => {
                execution_context
                    .generator
                    .as_mut()
                    .unwrap()
                    .generator_state = GeneratorState::Completed;
            }
        };

        let result_value = result.unwrap_or(Value::Undefined);
        return create_iterator_result_object(result_value, true);
    };

    generator.generator_context = Rc::downgrade(&ctx);
}

pub fn create_iterator_from_closure(
    closure: Box<dyn Fn() -> Option<Value>>,
    brand: Option<String>,
    prototype: Object,
) -> Generator {
    let mut generator = Generator {
        parent: ordinary_object_create(Some(prototype), vec![]),
        generator_state: GeneratorState::SuspendedStart,
        generator_context: Weak::new(),
        generator_brand: brand,
    };

    let callee_ctx = Rc::new(RefCell::new(ExecutionContext::Generic(
        GenericExecutionContext {
            function: None,
            generator: None,
            realm: current_realm(),
            script_or_module: None,
        },
    )));

    push_execution_context(callee_ctx.clone());
    generator_start(&mut generator, closure);
    pop_execution_context();

    generator
}
