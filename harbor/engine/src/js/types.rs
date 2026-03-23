pub mod completion_record {
    use std::fmt::Debug;

    pub type UNUSED = ();

    #[derive(Debug)]
    pub enum CompletionRecordError {
        Unused,

        ReferenceError,
        TypeError,
        RangeErorr,

        Misc(String),
    }

    #[derive(Debug)]
    pub enum CompletionRecordKind {
        Normal,
        Return,
        Throw,
        Break,
        Continue,
    }

    #[derive(Debug)]
    pub struct CompletionRecord<T: Debug> {
        pub kind: CompletionRecordKind,
        pub value: T,
        pub target: Option<String>,
    }

    impl<T: Debug> CompletionRecord<T> {
        pub fn unwrapped(&self) -> &T {
            &self.value
        }
    }

    #[allow(non_snake_case)]
    pub fn CompletionRecordNormal<T: Debug>(value: T) -> CompletionRecord<T> {
        CompletionRecord {
            kind: CompletionRecordKind::Normal,
            value,
            target: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn CompletionRecordThrow<T: Debug>(value: T) -> CompletionRecord<T> {
        CompletionRecord {
            kind: CompletionRecordKind::Throw,
            value,
            target: None,
        }
    }
}
