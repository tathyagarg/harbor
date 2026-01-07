#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::fmt::Debug;

use crate::font::ttf::otf_dtypes::*;

trait FromBeBytes {
    fn from_data(bytes: &[u8]) -> Self;
}

impl FromBeBytes for uint16 {
    fn from_data(bytes: &[u8]) -> Self {
        uint16::from_be_bytes(bytes[..2].try_into().unwrap())
    }
}

impl FromBeBytes for uint32 {
    fn from_data(bytes: &[u8]) -> Self {
        uint32::from_be_bytes(bytes[..4].try_into().unwrap())
    }
}

impl FromBeBytes for int16 {
    fn from_data(bytes: &[u8]) -> Self {
        int16::from_be_bytes(bytes[..2].try_into().unwrap())
    }
}

fn interpret_language(platform_id: uint16, language: uint16) -> uint16 {
    if platform_id == (PlatformID::Macintosh as uint16) {
        if language == (EncodingID_Macintosh::Roman as uint16) {
            return 0;
        }

        return language + 1;
    } else {
        0
    }
}

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

#[repr(u16)]
#[derive(Debug)]
pub enum PlatformID {
    Unicode = 0,
    Macintosh = 1,
    #[deprecated(note = "ISO encodings are deprecated")]
    ISO = 2,
    Windows = 3,
    Custom = 4,
}

#[repr(u16)]
#[derive(Debug)]
pub enum EncodingID_Unicode {
    #[deprecated(note = "Unicode 1.0 is deprecated")]
    Unicode_1_0 = 0,
    #[deprecated(note = "Unicode 1.1 is deprecated")]
    Unicode_1_1 = 1,
    #[deprecated(note = "ISO 10646 is deprecated")]
    ISO_10646 = 2,
    Unicode_2_0_BMP = 3,
    Unicode_2_0_Full = 4,
    Unicode_Variation_Sequences = 5,
    Unicode_Full_Support = 6,
}

#[repr(u16)]
#[derive(Debug)]
pub enum EncodingID_Macintosh {
    Roman = 0,
    Japanese = 1,
    Chinese_Traditional = 2,
    Korean = 3,
    Arabic = 4,
    Hebrew = 5,
    Greek = 6,
    Russian = 7,
    RSymbol = 8,
    Devanagari = 9,
    Gurmukhi = 10,
    Gujarati = 11,
    Oriya = 12,
    Bengali = 13,
    Tamil = 14,
    Telugu = 15,
    Kannada = 16,
    Malayalam = 17,
    Sinhalese = 18,
    Burmese = 19,
    Khmer = 20,
    Thai = 21,
    Laotian = 22,
    Georgian = 23,
    Armenian = 24,
    Chinese_Simplified = 25,
    Tibetan = 26,
    Mongolian = 27,
    Geez = 28,
    Slavic = 29,
    Vietnamese = 30,
    Sindhi = 31,
    Uninterpreted_Script = 32,
}

#[repr(u16)]
#[derive(Debug)]
pub enum EncodingID_ISO {
    SevenBitASCII = 0,
    ISO10646 = 1,
    ISO8859_1 = 2,
}

#[repr(u16)]
#[derive(Debug)]
pub enum EncodingID_Windows {
    Symbol = 0,
    Unicode_BMP = 1,
    ShiftJIS = 2,
    PRC = 3,
    Big5 = 4,
    Wansung = 5,
    Johab = 6,
    Unicode_Full = 10,
}

#[derive(Debug)]
pub enum EncodingID {
    Unicode(EncodingID_Unicode),
    Macintosh(EncodingID_Macintosh),
    ISO(EncodingID_ISO),
    Windows(EncodingID_Windows),
    Custom(uint16),
}

#[derive(Clone)]
pub struct CMAPEncodingRecord {
    /// Platform ID.
    pub platform_id: uint16,

    /// Platform-specific encoding ID.
    pub encoding_id: uint16,

    /// Byte offset from beginning of table to the subtable for this encoding.
    pub subtable_offset: Offset32,
}

impl Debug for CMAPEncodingRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CMAPEncodingRecord")
            .field("platform_id", &self.platform_id())
            .field("encoding_id", &self.encoding_id())
            .field("subtable_offset", &self.subtable_offset)
            .finish()
    }
}

impl CMAPEncodingRecord {
    pub fn platform_id(&self) -> PlatformID {
        if self.platform_id > 4 {
            panic!("Unknown Platform ID: {}", self.platform_id);
        }

        unsafe { std::mem::transmute(self.platform_id) }
    }

    pub fn encoding_id(&self) -> EncodingID {
        match self.platform_id() {
            PlatformID::Unicode => {
                if self.encoding_id > 6 {
                    panic!("Unknown Unicode Encoding ID: {}", self.encoding_id);
                }

                EncodingID::Unicode(unsafe { std::mem::transmute(self.encoding_id) })
            }
            PlatformID::Macintosh => {
                if self.encoding_id > 32 {
                    panic!("Unknown Macintosh Encoding ID: {}", self.encoding_id);
                }

                EncodingID::Macintosh(unsafe { std::mem::transmute(self.encoding_id) })
            }
            PlatformID::ISO => {
                if self.encoding_id > 2 {
                    panic!("Unknown ISO Encoding ID: {}", self.encoding_id);
                }

                EncodingID::ISO(unsafe { std::mem::transmute(self.encoding_id) })
            }
            PlatformID::Windows => {
                if self.encoding_id > 10 {
                    panic!("Unknown Windows Encoding ID: {}", self.encoding_id);
                }

                EncodingID::Windows(unsafe { std::mem::transmute(self.encoding_id) })
            }
            PlatformID::Custom => EncodingID::Custom(self.encoding_id),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CMAPSubtable0 {
    /// This is the length in bytes of the subtable.
    length: uint16,

    language: uint16,

    /// An array that maps character codes to glyph index values.
    glyph_id_array: [u8; 256],
}

impl Default for CMAPSubtable0 {
    fn default() -> Self {
        CMAPSubtable0 {
            length: 0,
            language: 0,
            glyph_id_array: [0; 256],
        }
    }
}

#[derive(Clone)]
pub struct CMAPSubtable4 {
    /// This is the length of the subtable.
    length: uint16,

    language: uint16,

    /// 2 × segCount.
    seg_count_x2: uint16,

    /// Maximum power of 2 less than or equal to segCount, times 2
    /// ((2**floor(log2(segCount))) * 2, where “**” is an exponentiation operator)
    search_range: uint16,

    /// Log2 of the maximum power of 2 less than or equal to segCount
    /// (log2(searchRange/2), which is equal to floor(log2(segCount)))
    entry_selector: uint16,

    /// segCount times 2, minus searchRange
    /// ((segCount * 2) - searchRange)
    range_shift: uint16,

    /// End characterCode for each segment, last=0xFFFF.
    end_code: Vec<uint16>,

    /// Set to 0.
    /// reserved_pad: uint16,

    /// Start character code for each segment.
    start_code: Vec<uint16>,

    /// Delta for all character codes in segment.
    id_delta: Vec<int16>,

    /// Offsets into glyphIdArray or 0
    id_range_offset: Vec<uint16>,

    /// Glyph index array (arbitrary length)
    glyph_id_array: Vec<uint16>,
}

impl Debug for CMAPSubtable4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CMAPSubtable4")
            .field("length", &self.length)
            .field("language", &self.language)
            .field("seg_count_x2", &self.seg_count_x2)
            .field("search_range", &self.search_range)
            .field("entry_selector", &self.entry_selector)
            .field("range_shift", &self.range_shift)
            .field("end_code", &&self.end_code[..4])
            .field("start_code", &&self.start_code[..4])
            .field("id_delta", &&self.id_delta[..4])
            .field("id_range_offset", &&self.id_range_offset[..4])
            .field("glyph_id_array_length", &self.glyph_id_array.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum CMAPSubtable {
    Format0(CMAPSubtable0),
    Format2,
    Format4(CMAPSubtable4),
    Format6,
    Format8,
    Format10,
    Format12,
    Format13,
    Format14,
}

#[derive(Clone, Default, Debug)]
pub struct CMAPTable {
    /// Table version number (0).
    pub version: uint16,

    /// Number of encoding tables that follow.
    pub num_tables: uint16,

    pub encoding_records: Vec<CMAPEncodingRecord>,

    pub subtables: Vec<CMAPSubtable>,
}

impl CMAPTable {
    pub fn new(version: uint16, num_tables: uint16) -> CMAPTable {
        CMAPTable {
            version,
            num_tables,
            encoding_records: Vec::with_capacity(num_tables as usize),
            subtables: Vec::new(),
        }
    }

    pub fn push_encoding_record(&mut self, record: CMAPEncodingRecord) {
        if self.encoding_records.len() == self.num_tables as usize {
            panic!(
                "Trying to add encoding record when required required no. of records has been reached:\n--> Required: {}\n--> Records: {:?}",
                self.num_tables, self.encoding_records
            )
        }

        self.encoding_records.push(record);
    }
}

#[derive(Clone)]
pub enum TableRecordData {
    CMAP(CMAPTable),
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
            b"cmap" => TableRecordData::CMAP(CMAPTable::default()),
            _ => TableRecordData::Raw(Vec::new()),
        }
    }

    pub fn from_tag_data(tag: Tag, data: &[u8]) -> TableRecordData {
        match &tag {
            b"cmap" => {
                let version = uint16::from_data(&data[0..2]);
                let num_tables = uint16::from_data(&data[2..4]);

                let mut cmap_table = CMAPTable::new(version, num_tables);

                let mut offset = 4;

                for _ in 0..num_tables {
                    let platform_id = uint16::from_data(&data[offset..]);
                    let encoding_id = uint16::from_data(&data[offset + 2..]);
                    let subtable_offset = uint32::from_data(&data[offset + 4..]);

                    cmap_table.push_encoding_record(CMAPEncodingRecord {
                        platform_id,
                        encoding_id,
                        subtable_offset,
                    });

                    offset += 8;
                }

                for encoding in &cmap_table.encoding_records {
                    let subtable_start = encoding.subtable_offset as usize;
                    let format = uint16::from_data(&data[subtable_start..]);

                    let subtable = match format {
                        0 => CMAPSubtable::Format0({
                            let length = uint16::from_data(&data[subtable_start + 2..]);

                            let language = interpret_language(
                                encoding.platform_id,
                                uint16::from_data(&data[subtable_start + 4..]),
                            );

                            let mut glyph_id_array = [0u8; 256];
                            glyph_id_array
                                .copy_from_slice(&data[subtable_start + 6..subtable_start + 262]);

                            CMAPSubtable0 {
                                length,
                                language,
                                glyph_id_array,
                            }
                        }),
                        2 => CMAPSubtable::Format2,
                        4 => CMAPSubtable::Format4({
                            let length = uint16::from_data(&data[subtable_start + 2..]);

                            let language = interpret_language(
                                encoding.platform_id,
                                uint16::from_data(&data[subtable_start + 4..]),
                            );

                            let seg_count_x2 = uint16::from_data(&data[subtable_start + 6..]);
                            let search_range = uint16::from_data(&data[subtable_start + 8..]);
                            let entry_selector = uint16::from_data(&data[subtable_start + 10..]);
                            let range_shift = uint16::from_data(&data[subtable_start + 12..]);

                            let seg_count = seg_count_x2 / 2;

                            let mut end_code = Vec::with_capacity(seg_count as usize);
                            let mut start_code = Vec::with_capacity(seg_count as usize);
                            let mut id_delta = Vec::with_capacity(seg_count as usize);
                            let mut id_range_offset = Vec::with_capacity(seg_count as usize);

                            let mut offset = subtable_start + 14;

                            for _ in 0..seg_count {
                                end_code.push(uint16::from_data(&data[offset..]));
                                offset += 2;
                            }

                            // reserved_pad
                            offset += 2;

                            for _ in 0..seg_count {
                                start_code.push(uint16::from_data(&data[offset..]));
                                offset += 2;
                            }

                            for _ in 0..seg_count {
                                id_delta.push(int16::from_data(&data[offset..]));
                                offset += 2;
                            }

                            for _ in 0..seg_count {
                                id_range_offset.push(uint16::from_data(&data[offset..]));
                                offset += 2;
                            }

                            let glyph_id_array_length =
                                (length as usize) - (offset - subtable_start);
                            let mut glyph_id_array = Vec::with_capacity(glyph_id_array_length / 2);

                            for _ in 0..(glyph_id_array_length / 2) {
                                glyph_id_array.push(uint16::from_data(&data[offset..]));
                                offset += 2;
                            }

                            CMAPSubtable4 {
                                length,
                                language,
                                seg_count_x2,
                                search_range,
                                entry_selector,
                                range_shift,
                                end_code,
                                start_code,
                                id_delta,
                                id_range_offset,
                                glyph_id_array,
                            }
                        }),
                        6 => CMAPSubtable::Format6,
                        8 => CMAPSubtable::Format8,
                        10 => CMAPSubtable::Format10,
                        12 => CMAPSubtable::Format12,
                        13 => CMAPSubtable::Format13,
                        14 => CMAPSubtable::Format14,
                        _ => panic!("Unknown CMAP subtable format: {}", format),
                    };

                    // For simplicity, we store the subtable as raw data for now
                    cmap_table.subtables.push(subtable);
                }

                TableRecordData::CMAP(cmap_table)
            }
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
