// pub fn array_create(length: u32, )

pub mod array {
    use crate::js::{
        operations::canonical_numeric_index_string,
        types::completion_record::CompletionRecord,
        values::object::{ArrayObject, PropertyDescriptor, PropertyKey},
    };

    impl ArrayObject {
        pub fn define_own_property(
            &mut self,
            key: PropertyKey,
            desc: PropertyDescriptor,
        ) -> Result<CompletionRecord<bool>, CompletionRecord<()>> {
            if key == "length" {
                return array_set_length(self, desc);
            }

            if let PropertyKey::String(s) = &key
                && canonical_numeric_index_string(s).is_some()
            {}

            todo!()
        }
    }

    pub fn array_set_length(
        array: &mut ArrayObject,
        desc: PropertyDescriptor,
    ) -> Result<CompletionRecord<bool>, CompletionRecord<()>> {
        if let PropertyDescriptor::Data { .. } = &desc {
            // return
        }

        todo!()
    }
}
