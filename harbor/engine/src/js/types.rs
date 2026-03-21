pub mod completion_record {
    pub type UNUSED = ();

    pub enum CompletionRecordError {
        Unused,

        ReferenceError(String),
    }

    pub enum CompletionRecordKind {
        Normal,
        Return,
        Throw,
        Break,
        Continue,
    }

    pub struct CompletionRecord<T> {
        pub kind: CompletionRecordKind,
        pub value: T,
        pub target: Option<String>,
    }

    #[allow(non_snake_case)]
    pub fn CompletionRecordNormal<T>(value: T) -> CompletionRecord<T> {
        CompletionRecord {
            kind: CompletionRecordKind::Normal,
            value,
            target: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn CompletionRecordThrow<T>(value: T) -> CompletionRecord<T> {
        CompletionRecord {
            kind: CompletionRecordKind::Throw,
            value,
            target: None,
        }
    }
}
