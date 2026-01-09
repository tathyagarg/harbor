#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{TableTrait, cmap, head, hhea, hmtx, loca, maxp, name, os2, post};

#[derive(Clone)]
pub enum TableRecordData {
    CMAP(cmap::CMAPTable),
    Head(head::HeaderTable),
    HHea(hhea::HHeaTable),
    HMtx(hmtx::HMtxTable),
    MaxP(maxp::MaxPTable),
    Name(name::NameTable),
    OS2(os2::OS2Table),
    Post(post::PostTable),
    Loca(loca::LocaTable),
    Raw(Vec<u8>),
}

impl Debug for TableRecordData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableRecordData::CMAP(cmap_table) => cmap_table.fmt(f),
            TableRecordData::Head(head_table) => head_table.fmt(f),
            TableRecordData::HHea(hhea_table) => hhea_table.fmt(f),
            TableRecordData::HMtx(hmtx_table) => hmtx_table.fmt(f),
            TableRecordData::MaxP(maxp_table) => maxp_table.fmt(f),
            TableRecordData::Name(name_table) => name_table.fmt(f),
            TableRecordData::OS2(os2_table) => os2_table.fmt(f),
            TableRecordData::Post(post_table) => post_table.fmt(f),
            TableRecordData::Loca(loca_table) => loca_table.fmt(f),
            TableRecordData::Raw(raw_data) => f
                .debug_struct("TableRecordData::Raw")
                .field("data_length", &raw_data.len())
                .finish(),
        }
    }
}

impl TableRecordData {
    pub fn from_tag_data(tag: Tag, data: &[u8], table_dir: &TableDirectory) -> TableRecordData {
        match &tag {
            b"cmap" => TableRecordData::CMAP(cmap::CMAPTable::parse(data, None)),
            b"head" => TableRecordData::Head(head::HeaderTable::parse(data, None)),
            b"hhea" => TableRecordData::HHea(hhea::HHeaTable::parse(data, None)),
            b"hmtx" => TableRecordData::HMtx({
                let mut hmtx_table = hmtx::HMtxTable::default()
                    .set_num_h_metrics(
                        table_dir
                            ._hhea_num_h_metrics
                            .expect("Number of hMetrics not set in TableDirectory."),
                    )
                    .set_num_glyphs(
                        table_dir
                            ._maxp_num_glyphs
                            .expect("Number of glyphs not set in TableDirectory."),
                    );

                hmtx_table.construct(data);
                hmtx_table
            }),
            b"maxp" => TableRecordData::MaxP(maxp::MaxPTable::parse(data, None)),
            b"name" => TableRecordData::Name(name::NameTable::parse(data, None)),
            b"OS/2" => TableRecordData::OS2({
                let mut os2_table = os2::OS2Table::Interim(
                    table_dir
                        ._head_mac_style
                        .expect("macStyle not set in TableDirectory."),
                );

                os2_table.construct(data);
                os2_table
            }),
            b"post" => TableRecordData::Post(post::PostTable::parse(data, None)),
            b"loca" => TableRecordData::Loca({
                let mut loca_table = loca::LocaTable::Interim((
                    table_dir
                        ._head_index_to_loc_format
                        .expect("indexToLocFormat not set in TableDirectory."),
                    table_dir
                        ._maxp_num_glyphs
                        .expect("Number of glyphs not set in TableDirectory.")
                        as uint16,
                ));

                loca_table.construct(data);
                loca_table
            }),
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
            .field("checksum", &format_args!("0x{:08X}", self.checksum))
            .field("offset", &self.offset)
            .field("length", &self.length)
            .field("_data", &self._data)
            .finish()
    }
}

impl TableRecord {
    pub fn new(
        table_tag: Tag,
        offset: Offset32,
        length: uint32,
        raw_data: &[u8],
        table_dir: &TableDirectory,
    ) -> TableRecord {
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
            table_dir,
        )
    }

    pub fn has_unmet_requirements(tag: Tag) -> Option<Box<dyn Fn(&TableDirectory) -> bool>> {
        match &tag {
            b"hmtx" => Some(Box::new(|table_dir: &TableDirectory| {
                table_dir._hhea_num_h_metrics.is_some() && table_dir._maxp_num_glyphs.is_some()
            })),
            b"OS/2" => Some(Box::new(|table_dir: &TableDirectory| {
                table_dir._head_mac_style.is_some()
            })),
            b"loca" => Some(Box::new(|table_dir: &TableDirectory| {
                table_dir._head_index_to_loc_format.is_some()
                    && table_dir._maxp_num_glyphs.is_some()
            })),
            _ => None,
        }
    }

    pub fn new_from_table_data(
        table_tag: Tag,
        offset: Offset32,
        length: uint32,
        table_data: &[u8],
        table_dir: &TableDirectory,
    ) -> TableRecord {
        TableRecord {
            table_tag,
            offset,
            length,
            checksum: 0,
            _data: TableRecordData::from_tag_data(table_tag, table_data, table_dir),
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

    _maxp_num_glyphs: Option<usize>,
    _hhea_num_h_metrics: Option<usize>,
    _head_mac_style: Option<uint16>,
    _head_index_to_loc_format: Option<int16>,

    _deferred_parse_queue: Vec<(Tag, Offset32, uint32, Box<dyn Fn(&TableDirectory) -> bool>)>,
}

impl Debug for TableDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableDirectory")
            .field("sfnt_version", &format_args!("{:08X}", self.sfnt_version))
            .field("num_tables", &self.num_tables)
            .field("search_range", &self.search_range)
            .field("entry_selector", &self.entry_selector)
            .field("range_shift", &self.range_shift)
            .field("table_records", &self.table_records)
            .finish()
    }
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
            _maxp_num_glyphs: None,
            _hhea_num_h_metrics: None,
            _head_mac_style: None,
            _head_index_to_loc_format: None,
            _deferred_parse_queue: Vec::new(),
        }
    }

    pub fn has_table(&self, tag: Tag) -> bool {
        self.table_records
            .iter()
            .any(|record| record.table_tag == tag)
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
        let table_tag: [u8; 4] = data[record_offset..record_offset + 4].try_into().unwrap();

        // let checksum = u32::from_be_bytes(
        //     data[record_offset + 4..record_offset + 8]
        //         .try_into()
        //         .unwrap(),
        // );

        let offset = Offset32::from_data(&data[record_offset + 8..]);
        let length = uint32::from_data(&data[record_offset + 12..]);

        if let Some(req) = TableRecord::has_unmet_requirements(table_tag) {
            if !req(&table_directory) {
                table_directory
                    ._deferred_parse_queue
                    .push((table_tag, offset, length, req));
                record_offset += 16;
                continue;
            }
        }

        let table_record = TableRecord::new(table_tag, offset, length, data, &table_directory);
        table_directory.table_records.push(table_record);

        match &table_tag {
            b"hhea" => {
                if let TableRecordData::HHea(hhea_table) =
                    &table_directory.table_records.last().unwrap()._data
                {
                    table_directory._hhea_num_h_metrics =
                        Some(hhea_table.number_of_h_metrics as usize);
                }
            }
            b"maxp" => {
                if let TableRecordData::MaxP(maxp_table) =
                    &table_directory.table_records.last().unwrap()._data
                {
                    match maxp_table {
                        maxp::MaxPTable::V0_5(table_v0_5) => {
                            table_directory._maxp_num_glyphs = Some(table_v0_5.num_glyphs as usize);
                        }
                        maxp::MaxPTable::V1_0(table_v1_0) => {
                            table_directory._maxp_num_glyphs = Some(table_v1_0.num_glyphs as usize);
                        }
                    }
                }
            }
            b"head" => {
                if let TableRecordData::Head(head_table) =
                    &table_directory.table_records.last().unwrap()._data
                {
                    table_directory._head_mac_style = Some(head_table.mac_style);
                    table_directory._head_index_to_loc_format =
                        Some(head_table.index_to_loc_format);
                }
            }
            _ => {}
        }

        let mut recorded_updates = true;
        while recorded_updates {
            recorded_updates = false;

            'req_update: for (i, (tag, offset, length, req)) in
                table_directory._deferred_parse_queue.iter().enumerate()
            {
                if req(&table_directory) {
                    recorded_updates = true;

                    let table_record =
                        TableRecord::new(*tag, *offset, *length, data, &table_directory);

                    table_directory.table_records.push(table_record);
                    _ = table_directory._deferred_parse_queue.remove(i);

                    break 'req_update;
                }
            }
        }

        record_offset += 16;
    }

    table_directory
}
