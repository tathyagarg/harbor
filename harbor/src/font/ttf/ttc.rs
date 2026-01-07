#![allow(non_camel_case_types)]
#![allow(dead_code)]

use crate::font::ttf::otf_dtypes::*;

pub struct TTCHeader_v1_0 {
    /// Font Collection ID string: 'ttcf' (used for fonts with CFF or CFF2 outlines,
    /// as well as TrueType outlines)
    pub ttc_tag: Tag,

    /// Major version of the TTCHeader, = 1.
    pub major_version: uint16,

    /// Minor version of the TTCHeader, = 0.
    pub minor_version: uint16,

    /// Number of fonts in TTC
    pub num_fonts: uint32,

    /// Array of offsets to the TableDirectory for each font from the beginning of the file
    table_directory_offsets: Vec<Offset32>,
}

#[derive(Default)]
pub struct TTCHeader_v2_0 {
    /// Font Collection ID string: 'ttcf' (used for fonts with CFF or CFF2 outlines,
    /// as well as TrueType outlines)
    pub ttc_tag: Tag,

    /// Major version of the TTCHeader, = 2.
    pub major_version: uint16,

    /// Minor version of the TTCHeader, = 0.
    pub minor_version: uint16,

    /// Number of fonts in TTC
    pub num_fonts: uint32,

    /// Array of offsets to the TableDirectory for each font from the beginning of the file
    table_directory_offsets: Vec<Offset32>,

    /// Tag indicating that a DSIG table exists, 0x44534947 ('DSIG') (null if no signature)
    pub dsig_tag: Option<uint32>,

    /// The length (in bytes) of the DSIG table (null if no signature)
    pub dsig_length: Option<uint32>,

    /// The offset (in bytes) of the DSIG table from the beginning of the TTC file (null if no signature)
    pub dsig_offset: Option<uint32>,
}

pub enum TTCHeader {
    v1_0(TTCHeader_v1_0),
    v2_0(TTCHeader_v2_0),
}

impl TTCHeader {
    pub fn new(
        ttc_tag: Tag,
        major_version: uint16,
        minor_version: uint16,
        num_fonts: uint32,
    ) -> TTCHeader {
        match (major_version, minor_version) {
            (1, 0) => TTCHeader::v1_0(TTCHeader_v1_0 {
                ttc_tag,
                major_version,
                minor_version,
                num_fonts,
                table_directory_offsets: vec![],
            }),
            (2, 0) => TTCHeader::v2_0(TTCHeader_v2_0 {
                ttc_tag,
                major_version,
                minor_version,
                num_fonts,
                ..Default::default()
            }),
            _ => panic!("Unknown TTC version: {}.{}", major_version, minor_version),
        }
    }

    pub fn table_directory_offsets(&self) -> &Vec<Offset32> {
        match self {
            TTCHeader::v1_0(ttc_v1) => &ttc_v1.table_directory_offsets,
            TTCHeader::v2_0(ttc_v2) => &ttc_v2.table_directory_offsets,
        }
    }

    pub fn table_directory_offsets_mut(&mut self) -> &mut Vec<Offset32> {
        match self {
            TTCHeader::v1_0(ttc_v1) => &mut ttc_v1.table_directory_offsets,
            TTCHeader::v2_0(ttc_v2) => &mut ttc_v2.table_directory_offsets,
        }
    }

    pub fn num_fonts(&self) -> uint32 {
        match self {
            TTCHeader::v1_0(ttc_v1) => ttc_v1.num_fonts,
            TTCHeader::v2_0(ttc_v2) => ttc_v2.num_fonts,
        }
    }

    pub fn push_table_directory_offset(&mut self, offset: Offset32) {
        if self.table_directory_offsets().len() == self.num_fonts() as usize {
            panic!(
                "Trying to add table directory offset when required required no. of offsets has been reached:\n--> Required: {}\n--> Offsets: {:?}",
                self.num_fonts(),
                self.table_directory_offsets()
            )
        }

        self.table_directory_offsets_mut().push(offset);
    }

    pub fn with_dsig_tag(mut self, dsig_tag: uint32) -> Self {
        if let TTCHeader::v2_0(ref mut ttc_v2) = self {
            ttc_v2.dsig_tag = Some(dsig_tag);
            return self;
        }

        panic!("Tried to add dsig tag: {} to non-TTCHeader v2.0", dsig_tag);
    }

    pub fn with_dsig_length(mut self, dsig_length: uint32) -> Self {
        if let TTCHeader::v2_0(ref mut ttc_v2) = self {
            ttc_v2.dsig_length = Some(dsig_length);
            return self;
        }

        panic!(
            "Tried to add dsig length: {} to non-TTCHeader v2.0",
            dsig_length
        );
    }

    pub fn with_dsig_offset(mut self, dsig_offset: uint32) -> Self {
        if let TTCHeader::v2_0(ref mut ttc_v2) = self {
            ttc_v2.dsig_offset = Some(dsig_offset);
            return self;
        }

        panic!(
            "Tried to add dsig offset: {} to non-TTCHeader v2.0",
            dsig_offset
        );
    }
}

pub fn parse_ttc(data: &[u8]) {
    println!(
        "Parsing TrueType Collection (TTC) data...: {:?}",
        &data[0..4]
    );
}
