#![allow(non_camel_case_types)]
// Is this a hackclub #meta reference?
// https://hackclub.enterprise.slack.com/archives/C0188CY57PZ

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

#[derive(Clone)]
pub enum MetaDataMapDataKind {
    Text(String),
    Binary(Vec<uint8>),
}

impl Debug for MetaDataMapDataKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaDataMapDataKind::Text(s) => f.debug_tuple("Text").field(s).finish(),
            MetaDataMapDataKind::Binary(b) => f
                .debug_tuple("Binary")
                .field(&b.len())
                .field(&b.iter().take(10).collect::<Vec<&uint8>>())
                .finish(),
        }
    }
}

#[derive(Clone)]
pub struct MetaDataMap {
    /// A tag indicating the type of metadata.
    pub tag: Tag,

    /// Offset in bytes from the beginning of the metadata table to the data for this tag.
    pub data_offset: Offset32,

    /// Length of the data, in bytes. The data is not required to be padded to any byte boundary.
    pub data_length: uint32,

    _data: Option<MetaDataMapDataKind>,
}

impl Debug for MetaDataMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetaDataMap")
            .field(
                "tag",
                &tag_as_str(&self.tag).unwrap_or("Invalid UTF-8".to_string()),
            )
            .field("data_offset", &self.data_offset)
            .field("data_length", &self.data_length)
            .field("_data", &self._data)
            .finish()
    }
}

#[derive(Clone)]
pub struct MetaTable {
    /// Version number of the metadata table — set to 1.
    /// pub version: uint32,

    /// Flags — currently unused; set to 0.
    /// pub flags: uint32,

    /// Not used; set to 0.
    /// pub reserved: uint32,

    /// The number of data maps in the table.
    pub data_map_count: uint32,

    /// Array of data map records.
    pub data_maps: Vec<MetaDataMap>,
}

fn get_tag_data(
    tag: &Tag,
    data: &[u8],
    data_offset: Offset32,
    data_length: uint32,
) -> Option<MetaDataMapDataKind> {
    let start = data_offset as usize;
    let end = start + data_length as usize;

    if end > data.len() {
        return None;
    }

    let tag_str = tag_as_str(tag).unwrap();

    match tag_str.as_str() {
        "dlng" | "slng" => {
            let text_data = &data[start..end];
            match str::from_utf8(text_data) {
                Ok(s) => Some(MetaDataMapDataKind::Text(s.to_string())),
                Err(_) => None,
            }
        }
        _ => {
            let binary_data = data[start..end].to_vec();
            Some(MetaDataMapDataKind::Binary(binary_data))
        }
    }
}

impl TableTrait for MetaTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> Self {
        let data_map_count = uint32::from_data(&data[12..16]);

        let mut data_maps = Vec::with_capacity(data_map_count as usize);
        let mut offset = 16;

        for _ in 0..data_map_count {
            let tag: Tag = (&data[offset..offset + 4]).try_into().unwrap();
            let data_offset = Offset32::from_data(&data[offset + 4..offset + 8]);
            let data_length = uint32::from_data(&data[offset + 8..offset + 12]);

            let tag_data = get_tag_data(&tag, data, data_offset, data_length);

            data_maps.push(MetaDataMap {
                tag,
                data_offset,
                data_length,
                _data: tag_data,
            });

            offset += 12;
        }

        MetaTable {
            data_map_count,
            data_maps,
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("MetaTable does not require construction - simply use MetaTable::parse()");
    }
}

impl Debug for MetaTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetaTable")
            .field("data_map_count", &self.data_map_count)
            .field(
                "data_maps_preview",
                &self.data_maps.iter().take(5).collect::<Vec<&MetaDataMap>>(),
            )
            .finish()
    }
}
