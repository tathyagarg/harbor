pub mod cmap;
pub mod gasp;
pub mod glyf;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod loca;
pub mod maxp;
pub mod meta;
pub mod name;
pub mod os2;
pub mod post;

use crate::font::otf_dtypes::*;

pub enum ParseContext {
    OS2(uint16),
    Loca((int16, uint16)),
}

pub trait TableTrait {
    fn parse(data: &[u8], ctx: Option<ParseContext>) -> Self
    where
        Self: Sized;

    fn construct(&mut self, data: &[u8]);
}

/// Small enough that I'm not creating a separate file for it
pub mod cvt {
    use super::{FromBeBytes, ParseContext, TableTrait};
    use crate::font::otf_dtypes::{FWORD, uint16};
    use std::fmt::Debug;

    #[derive(Clone)]
    pub struct CVTable {
        pub instructions: Vec<FWORD>,
        pub instruction_count: uint16,
    }

    impl Debug for CVTable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("CVTable")
                .field(
                    "instructions_preview",
                    &self.instructions.iter().take(10).collect::<Vec<&FWORD>>(),
                )
                .field("instruction_count", &self.instruction_count)
                .finish()
        }
    }

    impl TableTrait for CVTable {
        fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self {
            let instructions: Vec<FWORD> =
                data.chunks_exact(2).map(|b| FWORD::from_data(b)).collect();
            let length = instructions.len() as uint16;

            CVTable {
                instructions,
                instruction_count: length,
            }
        }

        fn construct(&mut self, data: &[u8]) {
            self.instructions = data.chunks_exact(2).map(|b| FWORD::from_data(b)).collect();
            self.instruction_count = self.instructions.len() as uint16;
        }
    }
}

/// Small enough that I'm not creating a separate file for it
pub mod fpgm {
    use super::{ParseContext, TableTrait};
    use crate::font::otf_dtypes::{uint8, uint16};
    use std::fmt::Debug;

    #[derive(Clone)]
    pub struct FPGMTable {
        pub instructions: Vec<uint8>,
        pub instruction_count: uint16,
    }

    impl Debug for FPGMTable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("FPGMTable")
                .field(
                    "instructions_preview",
                    &self.instructions.iter().take(10).collect::<Vec<&uint8>>(),
                )
                .field("instruction_count", &self.instruction_count)
                .finish()
        }
    }

    impl TableTrait for FPGMTable {
        fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self {
            let instructions: Vec<uint8> = data.iter().cloned().collect();
            let length = instructions.len() as uint16;

            FPGMTable {
                instructions,
                instruction_count: length,
            }
        }

        fn construct(&mut self, data: &[u8]) {
            self.instructions = data.iter().cloned().collect();
            self.instruction_count = self.instructions.len() as uint16;
        }
    }
}

/// Small enough that I'm not creating a separate file for it
pub mod prep {
    use super::{ParseContext, TableTrait};
    use crate::font::otf_dtypes::{uint8, uint16};
    use std::fmt::Debug;

    #[derive(Clone)]
    pub struct PrepTable {
        pub instructions: Vec<uint8>,
        pub instruction_count: uint16,
    }

    impl Debug for PrepTable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PrepTable")
                .field(
                    "instructions_preview",
                    &self.instructions.iter().take(10).collect::<Vec<&uint8>>(),
                )
                .field("instruction_count", &self.instruction_count)
                .finish()
        }
    }

    impl TableTrait for PrepTable {
        fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self {
            let instructions: Vec<uint8> = data.iter().cloned().collect();
            let length = instructions.len() as uint16;

            PrepTable {
                instructions,
                instruction_count: length,
            }
        }

        fn construct(&mut self, data: &[u8]) {
            self.instructions = data.iter().cloned().collect();
            self.instruction_count = self.instructions.len() as uint16;
        }
    }
}
