#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;
use std::ops::BitAnd;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

#[derive(Clone)]
pub struct HdmxDeviceRecord {
    /// Pixel size for following widths (as ppem).
    pub pixel_size: uint8,

    /// Maximum width.
    pub max_width: uint8,

    /// Array of widths (numGlyphs is from the 'maxp' table).
    pub widths: Vec<uint8>,
}

impl Debug for HdmxDeviceRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HdmxDeviceRecord")
            .field("pixel_size", &self.pixel_size)
            .field("max_width", &self.max_width)
            .field(
                "widths_preview",
                &self.widths.iter().take(10).collect::<Vec<&uint8>>(),
            )
            .finish()
    }
}

#[derive(Clone)]
pub struct HdmxTable {
    /// Table version number—set to 0.
    /// version: uint16,

    /// Number of device records.
    pub num_records: uint16,

    /// Size of a device record, 32-bit aligned.
    pub record_size: uint32,

    /// Array of device records.
    pub device_records: Vec<HdmxDeviceRecord>,
}

impl TableTrait for HdmxTable {
    fn parse(data: &[u8], ctx: Option<ParseContext>) -> Self {
        let num_glyphs = if let Some(ParseContext::Hdmx(n)) = ctx {
            n
        } else {
            panic!("HdmxTable parsing requires ParseContext::Hdmx with num_glyphs");
        };

        let mut offset = 2;

        let num_records = uint16::from_data(&data[offset..offset + 2]);
        offset += 2;

        let record_size = uint32::from_data(&data[offset..offset + 4]);
        offset += 4;

        let mut device_records = Vec::with_capacity(num_records as usize);

        for _ in 0..num_records {
            let pixel_size = data[offset];
            offset += 1;

            let max_width = data[offset];
            offset += 1;

            let mut widths = Vec::with_capacity(num_glyphs as usize);
            for _ in 0..num_glyphs as usize {
                widths.push(data[offset]);
                offset += 1;
            }

            // Align to 4-byte boundary
            let padding = (record_size as usize).bitand(3);
            if padding != 0 {
                offset += 4 - padding;
            }

            device_records.push(HdmxDeviceRecord {
                pixel_size,
                max_width,
                widths,
            });
        }

        HdmxTable {
            num_records,
            record_size,
            device_records,
        }
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!("HdmxTable does not need to be constructed - use HdmxTable::parse() instead");
    }
}

impl Debug for HdmxTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HdmxTable")
            .field("num_records", &self.num_records)
            .field("record_size", &self.record_size)
            .field(
                "device_records_preview",
                &self
                    .device_records
                    .iter()
                    .take(5)
                    .collect::<Vec<&HdmxDeviceRecord>>(),
            )
            .finish()
    }
}
