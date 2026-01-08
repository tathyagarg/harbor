#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{cmap, head};

#[derive(Clone)]
pub enum TableRecordData {
    CMAP(cmap::CMAPTable),
    Head(head::HeaderTable),
    Raw(Vec<u8>),
}

impl Debug for TableRecordData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableRecordData::CMAP(cmap_table) => f
                .debug_struct("TableRecordData::CMAP")
                .field("version", &cmap_table.version)
                .field("num_tables", &cmap_table.num_tables)
                .field("encoding_records", &cmap_table.encoding_records)
                .field("subtables", &cmap_table.subtables)
                .finish(),
            TableRecordData::Head(head_table) => head_table.fmt(f),
            TableRecordData::Raw(raw_data) => f
                .debug_struct("TableRecordData::Raw")
                .field("data_length", &raw_data.len())
                .finish(),
        }
    }
}

impl TableRecordData {
    pub fn from_tag(tag: Tag) -> TableRecordData {
        match &tag {
            b"cmap" => TableRecordData::CMAP(cmap::CMAPTable::default()),
            b"head" => TableRecordData::Head(head::HeaderTable::default()),
            _ => TableRecordData::Raw(Vec::new()),
        }
    }

    pub fn from_tag_data(tag: Tag, data: &[u8]) -> TableRecordData {
        match &tag {
            b"cmap" => TableRecordData::CMAP(cmap::CMAPTable::parse(data)),
            b"head" => TableRecordData::Head(head::HeaderTable::parse(data)),
            _ => TableRecordData::Raw(data.to_vec()),
        }
    }
}

#[derive(Clone)]
pub struct TableRecord {
    /// Table identifier.
    pub table_tag: Tag,

    /// Checksum for this table.
    checksum: uint32,

    /// Offset from beginning of font file.
    offset: Offset32,

    /// Length of this table.
    length: uint32,

    _data: TableRecordData,
}

impl Debug for TableRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableRecord")
            .field(
                "table_tag",
                &tag_as_str(self.table_tag).unwrap_or(String::from("Invalid Tag")),
            )
            .field("checksum", &format_args!("{:08X}", self.checksum))
            .field("offset", &self.offset)
            .field("length", &self.length)
            .field("_data", &self._data)
            .finish()
    }
}

impl TableRecord {
    pub fn new(table_tag: Tag, offset: Offset32, length: uint32, raw_data: &[u8]) -> TableRecord {
        if !is_valid_tag(table_tag) {
            panic!(
                "Invalid table tag: {:?}",
                table_tag.map(|b| b as char).iter().collect::<String>()
            );
        }

        TableRecord::new_from_table_data(
            table_tag,
            offset,
            length,
            &raw_data[offset as usize..(offset + length) as usize],
        )
    }

    pub fn new_from_table_data(
        table_tag: Tag,
        offset: Offset32,
        length: uint32,
        table_data: &[u8],
    ) -> TableRecord {
        TableRecord {
            table_tag,
            offset,
            length,
            checksum: 0,
            _data: TableRecordData::from_tag_data(table_tag, table_data),
        }
        .reassign_checksum(table_data)
    }

    fn reassign_checksum(mut self, table_data: &[u8]) -> Self {
        let mut sum: uint32 = 0;

        for chunk in table_data.chunks(4) {
            let mut word = [0u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum = sum.wrapping_add(uint32::from_be_bytes(word));
        }

        self.checksum = sum;
        self
    }

    pub fn recalculate_checksum(&self, data: &[u8]) -> uint32 {
        let mut sum: uint32 = 0;

        for chunk in data.chunks(4) {
            let mut word = [0u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum = sum.wrapping_add(uint32::from_be_bytes(word));
        }

        sum
    }
}

#[derive(Debug)]
pub struct TableDirectory {
    /// 0x00010000 or 0x4F54544F ('OTTO') — see below.
    pub sfnt_version: uint32,

    /// Number of tables.
    pub num_tables: uint16,

    /// Maximum power of 2 less than or equal to numTables, times 16 ((2**floor(log2(numTables))) * 16,
    /// where “**” is an exponentiation operator).
    pub search_range: uint16,

    /// Log2 of the maximum power of 2 less than or equal to numTables (log2(searchRange/16),
    /// which is equal to floor(log2(numTables))).
    pub entry_selector: uint16,

    /// numTables times 16, minus searchRange ((numTables * 16) - searchRange).
    pub range_shift: uint16,

    /// Table records array—one for each top-level table in the font.
    pub table_records: Vec<TableRecord>,
}

impl TableDirectory {
    pub fn new(
        sfnt_version: uint32,
        num_tables: uint16,
        search_range: uint16,
        entry_selector: uint16,
        range_shift: uint16,
    ) -> TableDirectory {
        TableDirectory {
            sfnt_version,
            num_tables,
            search_range,
            entry_selector,
            range_shift,
            table_records: Vec::with_capacity(num_tables as usize),
        }
    }
}

pub fn parse_table_directory(data: &[u8], offset: Option<usize>) -> TableDirectory {
    let start_offset = offset.unwrap_or(0);
    let sfnt_version = uint32::from_data(&data[start_offset..]);
    let num_tables = uint16::from_data(&data[start_offset + 4..]);

    let search_range = uint16::from_data(&data[start_offset + 6..]);
    let entry_selector = uint16::from_data(&data[start_offset + 8..]);
    let range_shift = uint16::from_data(&data[start_offset + 10..]);

    let mut table_directory = TableDirectory::new(
        sfnt_version,
        num_tables,
        search_range,
        entry_selector,
        range_shift,
    );

    let mut record_offset = start_offset + 12;

    for _ in 0..num_tables {
        let table_tag = &data[record_offset..record_offset + 4];

        // let checksum = u32::from_be_bytes(
        //     data[record_offset + 4..record_offset + 8]
        //         .try_into()
        //         .unwrap(),
        // );

        let offset = Offset32::from_be_bytes(
            data[record_offset + 8..record_offset + 12]
                .try_into()
                .unwrap(),
        );
        let length = uint32::from_be_bytes(
            data[record_offset + 12..record_offset + 16]
                .try_into()
                .unwrap(),
        );

        let table_record = TableRecord::new(table_tag.try_into().unwrap(), offset, length, data);

        table_directory.table_records.push(table_record);
        record_offset += 16;
    }

    table_directory
}
