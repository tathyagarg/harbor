use crate::js::{
    executable::environment::EnvironmentRecord,
    operations::to_object,
    types::completion_record::{
        CRKAbrupt, CompletionRecord, CompletionRecordError, CompletionRecordNormal,
    },
    values::{
        ReferenceOrValue, Value,
        object::{ObjectTrait, PropertyKey},
    },
};

#[derive(Debug, Clone)]
pub enum ReferenceBase {
    Unresolvable,
    Value(Value),
    EnvironmentRecord(EnvironmentRecord),
}

#[derive(Debug, Clone)]
pub enum ReferenceName {
    Value(Value),
    Private(()),
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub base: ReferenceBase,
    pub referenced_name: ReferenceName,
    pub strict: bool,
    pub this_value: Option<Value>,
}

pub fn is_property_reference(reference: &Reference) -> bool {
    if let ReferenceBase::Unresolvable = reference.base {
        return false;
    }

    if let ReferenceBase::EnvironmentRecord(_) = reference.base {
        return false;
    }

    return true;
}

pub fn is_unresolvable_reference(reference: &Reference) -> bool {
    if let ReferenceBase::Unresolvable = reference.base {
        return true;
    }

    return false;
}

pub fn is_super_reference(reference: &Reference) -> bool {
    reference.this_value.is_some()
}

pub fn is_private_reference(reference: &Reference) -> bool {
    if let ReferenceName::Private(_) = reference.referenced_name {
        return true;
    }

    return false;
}

pub fn get_value(
    val: ReferenceOrValue,
) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError, CRKAbrupt>> {
    if let ReferenceOrValue::Value(value) = val {
        return Ok(CompletionRecordNormal(value));
    }

    let reference = match val {
        ReferenceOrValue::Reference(reference) => reference,
        _ => unreachable!(),
    };

    if is_unresolvable_reference(&reference) {
        return Err(CompletionRecord {
            kind: CRKAbrupt::Throw,
            value: CompletionRecordError::ReferenceError,
            target: None,
        });
    }

    if is_property_reference(&reference) {
        let base = match &reference.base {
            ReferenceBase::Value(val) => val,
            _ => unreachable!(),
        };
        let maybe_base_object = to_object(base);
        if let Err(e) = maybe_base_object {
            return Err(CompletionRecord {
                kind: CRKAbrupt::Throw,
                value: e.value,
                target: None,
            });
        }

        let base_record = maybe_base_object.unwrap();
        let base_object = base_record.unwrapped();

        if is_private_reference(&reference) {
            todo!("Private reference")
        }

        if let ReferenceName::Value(val) = &reference.referenced_name
            && !val.is_property_key()
        {
            todo!("To property key")
        }

        let prop_key = PropertyKey::from(reference.referenced_name.clone());
        let res = base_object.get(&prop_key, &get_this_value(&reference));

        match res {
            Some(val) => return Ok(CompletionRecordNormal(val)),
            None => return Ok(CompletionRecordNormal(Value::Undefined)),
        }
    }

    let base_env = match &reference.base {
        ReferenceBase::EnvironmentRecord(env) => env,
        _ => unreachable!(),
    };

    let name = match &reference.referenced_name {
        ReferenceName::Value(val) => val.unwrap_string().unwrap(),
        _ => unreachable!(),
    };

    let res = base_env.get_binding_value(name, reference.strict);

    match res {
        Ok(rec) => return Ok(rec),
        Err(e) => {
            return Err(CompletionRecord {
                kind: CRKAbrupt::Throw,
                value: e.value,
                target: None,
            });
        }
    }
}

pub fn get_this_value(reference: &Reference) -> Value {
    if is_super_reference(reference) {
        return reference.this_value.clone().unwrap();
    }

    return match &reference.base {
        ReferenceBase::Value(val) => val.clone(),
        _ => Value::Undefined,
    };
}
