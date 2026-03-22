use crate::js::{
    types::completion_record::{CompletionRecord, CompletionRecordError},
    values::{
        Value,
        object::{Object, PropertyDescriptor, PropertyKey},
    },
};

// 10.4
pub mod exotics;

pub trait Ordinary {
    fn get_prototype_of(&self) -> CompletionRecord<Option<Object>>;
    fn _ordinary_get_prototype_of(&self) -> Option<Object>;

    fn set_prototype_of(&mut self, prototype: Option<Object>) -> CompletionRecord<bool>;
    fn _ordinary_set_prototype_of(&mut self, prototype: Option<Object>) -> bool;

    fn is_extensible(&self) -> CompletionRecord<bool>;
    fn _ordinary_is_extensible(&self) -> bool;

    fn prevent_extensions(&mut self) -> CompletionRecord<bool>;
    fn _ordinary_prevent_extensions(&mut self) -> bool;

    fn get_own_property(&self, key: PropertyKey) -> CompletionRecord<Option<PropertyDescriptor>>;
    fn _ordinary_get_own_property(&self, key: PropertyKey) -> Option<PropertyDescriptor>;

    fn define_own_property(
        &mut self,
        key: PropertyKey,
        desc: PropertyDescriptor,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;
    fn _ordinary_define_own_property(
        &mut self,
        key: PropertyKey,
        desc: PropertyDescriptor,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;
    fn is_compatible_property_descriptor(
        extensible: bool,
        desc: &PropertyDescriptor,
        current: Option<&PropertyDescriptor>,
    ) -> bool;
    fn validate_and_apply_property_descriptor(
        object: Option<&mut Object>,
        key: PropertyKey,
        extensible: bool,
        desc: &PropertyDescriptor,
        current: Option<&PropertyDescriptor>,
    ) -> bool;

    fn has_property(
        &self,
        key: PropertyKey,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;
    fn _ordinary_has_property(
        &self,
        key: PropertyKey,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    fn get(
        &self,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError>>;
    fn _ordinary_get(
        &self,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<CompletionRecord<Value>, CompletionRecord<CompletionRecordError>>;

    fn set(
        &mut self,
        key: PropertyKey,
        value: Value,
        receiver: Value,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;
    fn _ordinary_set(
        &mut self,
        key: PropertyKey,
        value: Value,
        receiver: Value,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;
    fn _ordinary_set_with_own_descriptor(
        &mut self,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        own_desc: Option<PropertyDescriptor>,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    fn delete(
        &mut self,
        key: PropertyKey,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;
    fn _ordinary_delete(
        &mut self,
        key: PropertyKey,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<CompletionRecordError>>;

    fn own_property_keys(&self) -> CompletionRecord<Vec<PropertyKey>>;
    fn _ordinary_own_property_keys(&self) -> Vec<PropertyKey>;
}
