pub mod completion_record {
    use std::fmt::Debug;

    pub type UNUSED = ();

    #[derive(Debug, Clone)]
    pub enum CompletionRecordError {
        Unused,

        ReferenceError,
        TypeError,
        RangeErorr,

        Misc(String),
    }

    #[derive(Debug)]
    pub struct CRKNormal;
    #[derive(Debug)]
    pub struct CRKReturn;
    #[derive(Debug)]
    pub struct CRKThrow;
    #[derive(Debug)]
    pub struct CRKBreak;
    #[derive(Debug)]
    pub struct CRKContinue;

    pub trait CRK {}

    impl CRK for CRKNormal {}
    impl CRK for CRKReturn {}
    impl CRK for CRKThrow {}
    impl CRK for CRKBreak {}
    impl CRK for CRKContinue {}

    #[derive(Debug)]
    pub enum CRKAbrupt {
        Return,
        Throw,
        Break,
        Continue,
    }

    impl CRK for CRKAbrupt {}

    // #[derive(Debug)]
    // pub enum CompletionRecordKind {
    //     Normal,
    //     Return,
    //     Throw,
    //     Break,
    //     Continue,
    // }

    #[derive(Debug)]
    pub struct CompletionRecord<T: Debug, K: CRK = CRKNormal> {
        pub kind: K,
        pub value: T,
        pub target: Option<String>,
    }

    impl<T: Debug, K: CRK> CompletionRecord<T, K> {
        pub fn unwrapped(&self) -> &T {
            &self.value
        }
    }

    #[allow(non_snake_case)]
    pub fn CompletionRecordNormal<T: Debug>(value: T) -> CompletionRecord<T, CRKNormal> {
        CompletionRecord {
            kind: CRKNormal,
            value,
            target: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn CompletionRecordThrow<T: Debug>(value: T) -> CompletionRecord<T, CRKThrow> {
        CompletionRecord {
            kind: CRKThrow,
            value,
            target: None,
        }
    }
}
