#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

pub enum NameID {
    Copyright = 0,
    FontFamily = 1,
    FontSubfamily = 2,
    UniqueFontIdentifier = 3,
    FullFontName = 4,
    VersionString = 5,
    PostscriptName = 6,
    Trademark = 7,
    Manufacturer = 8,
    Designer = 9,
    Description = 10,
    VendorURL = 11,
    DesignerURL = 12,
    LicenseDescription = 13,
    LicenseURL = 14,
    TypographicFamily = 16,
    TypographicSubfamily = 17,
    CompatibleFullFontName = 18,
    SampleText = 19,
    PostscriptCIDFindfontName = 20,
    WWSFamilyName = 21,
    WWSSubfamilyName = 22,
    LightBackgroundPalette = 23,
    DarkBackgroundPalette = 24,
    VariationsPostscriptNamePrefix = 25,
}

fn name_id_to_string(name_id: uint16) -> Option<&'static str> {
    match name_id {
        0 => Some("Copyright"),
        1 => Some("Font Family"),
        2 => Some("Font Subfamily"),
        3 => Some("Unique Font Identifier"),
        4 => Some("Full Font Name"),
        5 => Some("Version String"),
        6 => Some("Postscript Name"),
        7 => Some("Trademark"),
        8 => Some("Manufacturer"),
        9 => Some("Designer"),
        10 => Some("Description"),
        11 => Some("Vendor URL"),
        12 => Some("Designer URL"),
        13 => Some("License Description"),
        14 => Some("License URL"),
        16 => Some("Typographic Family"),
        17 => Some("Typographic Subfamily"),
        18 => Some("Compatible Full Font Name"),
        19 => Some("Sample Text"),
        20 => Some("Postscript CID Findfont Name"),
        21 => Some("WWS Family Name"),
        22 => Some("WWS Subfamily Name"),
        23 => Some("Light Background Palette"),
        24 => Some("Dark Background Palette"),
        25 => Some("Variations Postscript Name Prefix"),
        _ => None,
    }
}

#[derive(Clone)]
pub struct NameRecord {
    /// Platform ID.
    pub platform_id: uint16,

    /// Platform-specific encoding ID.
    pub encoding_id: uint16,

    /// Language ID.
    pub language_id: uint16,

    /// Name ID.
    pub name_id: uint16,

    /// String length (in bytes).
    pub length: uint16,

    /// String offset from start of storage area (in bytes).
    pub offset: Offset16,

    _data: String,
}

impl Debug for NameRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name_type = name_id_to_string(self.name_id)
            .unwrap_or("Unknown")
            .to_string();
        f.debug_struct("NameRecord")
            .field("platform_id", &self.platform_id)
            .field("encoding_id", &self.encoding_id)
            .field("language_id", &self.language_id)
            .field("name_id", &self.name_id)
            .field("name_type", &name_type)
            .field("length", &self.length)
            .field("offset", &self.offset)
            .field("data", &self._data)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct NameTable_v0 {
    /// Table version number (=0).
    /// version: uint16,

    /// Number of name records.
    pub count: uint16,

    /// Offset to start of string storage (from start of table).
    pub storage_offset: Offset16,

    /// The name records where count is the number of records.
    pub name_records: Vec<NameRecord>,
}

#[derive(Clone)]
pub enum NameTable {
    // TODO: Add support for version 1
    v0(NameTable_v0),
}

impl Debug for NameTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameTable::v0(table) => f
                .debug_struct("NameTable_v0")
                .field("count", &table.count)
                .field("storage_offset", &table.storage_offset)
                .field(
                    "name_records_preview",
                    &table
                        .name_records
                        .iter()
                        .take(5)
                        .collect::<Vec<&NameRecord>>(),
                )
                .finish(),
        }
    }
}

impl TableTrait for NameTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self
    where
        Self: Sized,
    {
        let version = uint16::from_data(&data[..2]);
        match version {
            0 => {
                let count = uint16::from_data(&data[2..4]);
                let storage_offset = Offset16::from_data(&data[4..6]);

                let mut name_records = Vec::new();

                for i in 0..count as usize {
                    let offset = 6 + i * 12;
                    let mut name_record = NameRecord {
                        platform_id: uint16::from_data(&data[offset..offset + 2]),
                        encoding_id: uint16::from_data(&data[offset + 2..offset + 4]),
                        language_id: uint16::from_data(&data[offset + 4..offset + 6]),
                        name_id: uint16::from_data(&data[offset + 6..offset + 8]),
                        length: uint16::from_data(&data[offset + 8..offset + 10]),
                        offset: Offset16::from_data(&data[offset + 10..offset + 12]),
                        _data: String::new(),
                    };

                    let raw_data = &data[(storage_offset as usize + name_record.offset as usize)
                        ..(storage_offset as usize
                            + name_record.offset as usize
                            + name_record.length as usize)];

                    name_record._data = match name_record.platform_id {
                        0 | 3 => String::from_utf16(
                            &raw_data
                                .chunks(2)
                                .map(|b| uint16::from_data(b))
                                .collect::<Vec<uint16>>(),
                        )
                        .unwrap_or_default(),
                        1 => raw_data.iter().map(|&b| b as char).collect::<String>(),
                        _ => String::from_utf8_lossy(raw_data).to_string(),
                    };

                    name_records.push(name_record);
                }

                NameTable::v0(NameTable_v0 {
                    count,
                    storage_offset,
                    name_records,
                })
            }
            _ => panic!("Unsupported NameTable version: {}", version),
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("NameTable does not require construction - simply use NameTable::parse()");
    }
}

impl NameTable {
    pub fn find_name_id(&self, name_id: uint16) -> Option<&str> {
        match self {
            NameTable::v0(table) => {
                for record in &table.name_records {
                    if record.name_id == name_id {
                        return Some(&record._data);
                    }
                }
                None
            }
        }
    }
}
