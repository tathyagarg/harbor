#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::ttf::{TableDirectory, parse_table_directory};

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

#[derive(Default, Debug)]
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
    pub dsig_tag: uint32,

    /// The length (in bytes) of the DSIG table (null if no signature)
    pub dsig_length: uint32,

    /// The offset (in bytes) of the DSIG table from the beginning of the TTC file (null if no signature)
    pub dsig_offset: uint32,
}

#[derive(Debug)]
pub enum TTCHeader {
    v1_0(TTCHeader_v1_0),
    v2_0(TTCHeader_v2_0),
}

impl Debug for TTCHeader_v1_0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TTCHeader_v1_0")
            .field(
                "ttc_tag",
                &tag_as_str(self.ttc_tag).unwrap_or(String::from("Invalid Tag")),
            )
            .field("major_version", &self.major_version)
            .field("minor_version", &self.minor_version)
            .field("num_fonts", &self.num_fonts)
            .field("table_directory_offsets", &self.table_directory_offsets)
            .finish()
    }
}

impl TTCHeader {
    pub fn new(
        ttc_tag: Tag,
        major_version: uint16,
        minor_version: uint16,
        num_fonts: uint32,
    ) -> TTCHeader {
        if !is_valid_tag(ttc_tag) {
            panic!(
                "Invalid TTC tag: {:?}",
                ttc_tag.map(|b| b as char).iter().collect::<String>()
            );
        }

        match (major_version, minor_version) {
            (1, 0) => TTCHeader::v1_0(TTCHeader_v1_0 {
                ttc_tag,
                major_version,
                minor_version,
                num_fonts,
                table_directory_offsets: Vec::with_capacity(num_fonts as usize),
            }),
            (2, 0) => TTCHeader::v2_0(TTCHeader_v2_0 {
                ttc_tag,
                major_version,
                minor_version,
                num_fonts,
                table_directory_offsets: Vec::with_capacity(num_fonts as usize),
                // ints become 0
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
            ttc_v2.dsig_tag = dsig_tag;
            return self;
        }

        panic!("Tried to add dsig tag: {} to non-TTCHeader v2.0", dsig_tag);
    }

    pub fn with_dsig_length(mut self, dsig_length: uint32) -> Self {
        if let TTCHeader::v2_0(ref mut ttc_v2) = self {
            ttc_v2.dsig_length = dsig_length;
            return self;
        }

        panic!(
            "Tried to add dsig length: {} to non-TTCHeader v2.0",
            dsig_length
        );
    }

    pub fn with_dsig_offset(mut self, dsig_offset: uint32) -> Self {
        if let TTCHeader::v2_0(ref mut ttc_v2) = self {
            ttc_v2.dsig_offset = dsig_offset;
            return self;
        }

        panic!(
            "Tried to add dsig offset: {} to non-TTCHeader v2.0",
            dsig_offset
        );
    }

    pub fn has_dsig(&self) -> bool {
        match self {
            TTCHeader::v1_0(_) => false,
            TTCHeader::v2_0(ttc_v2) => ttc_v2.dsig_tag != 0,
        }
    }
}

#[derive(Debug)]
pub struct TTCData {
    header: TTCHeader,
    table_directories: Vec<TableDirectory>,
}

pub fn parse_ttc_header(data: &[u8]) -> TTCHeader {
    let ttc_tag = &data[0..4];
    let major_version = uint16::from_be_bytes(data[4..6].try_into().unwrap());
    let minor_version = uint16::from_be_bytes(data[6..8].try_into().unwrap());
    let num_fonts = uint32::from_be_bytes(data[8..12].try_into().unwrap());

    let mut ttc_header = TTCHeader::new(
        ttc_tag.try_into().unwrap(),
        major_version,
        minor_version,
        num_fonts,
    );

    let mut offset = 12;

    for _ in 0..num_fonts {
        let table_dir_offset =
            Offset32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
        ttc_header.push_table_directory_offset(table_dir_offset);
        offset += 4;
    }

    if major_version == 2 && minor_version == 0 {
        let dsig_tag = uint32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let dsig_length = uint32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let dsig_offset = uint32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
        // offset not used again after this
        // offset += 4;

        ttc_header = ttc_header
            .with_dsig_tag(dsig_tag)
            .with_dsig_length(dsig_length)
            .with_dsig_offset(dsig_offset);
    }

    ttc_header
}

pub fn parse_ttc(data: &[u8]) -> TTCData {
    let ttc_header = parse_ttc_header(data);
    let mut table_directories = Vec::with_capacity(ttc_header.num_fonts() as usize);

    for table_dir_offset in ttc_header.table_directory_offsets() {
        let table_directory = parse_table_directory(data, Some(*table_dir_offset as usize));
        table_directories.push(table_directory);
    }

    TTCData {
        header: ttc_header,
        table_directories,
    }
}
