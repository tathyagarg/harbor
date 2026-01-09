#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::{ParseContext, TableTrait};

fn interpret_language(platform_id: uint16, language: uint16) -> uint16 {
    if platform_id == (PlatformID::Macintosh as uint16) {
        if language == (MacintoshEncodingID::Roman as uint16) {
            return 0;
        }

        return language + 1;
    } else {
        0
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
pub enum UnicodeEncodingID {
    #[deprecated(note = "Unicode 1.0 is deprecated")]
    Unicode_1_0 = 0,
    #[deprecated(note = "Unicode 1.1 is deprecated")]
    Unicode_1_1 = 1,
    #[deprecated(note = "ISO 10646 is deprecated")]
    ISO_10646 = 2,
    Unicode_2_0_BMP = 3,
    Unicode_2_0_Full = 4,
    UnicodeVariationSequences = 5,
    UnicodeFullSupport = 6,
}

#[repr(u16)]
#[derive(Debug)]
pub enum MacintoshEncodingID {
    Roman = 0,
    Japanese = 1,
    ChineseTraditional = 2,
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
    ChineseSimplified = 25,
    Tibetan = 26,
    Mongolian = 27,
    Geez = 28,
    Slavic = 29,
    Vietnamese = 30,
    Sindhi = 31,
    UninterpretedScript = 32,
}

#[repr(u16)]
#[derive(Debug)]
pub enum ISOEncodingID {
    SevenBitASCII = 0,
    ISO10646 = 1,
    ISO8859_1 = 2,
}

#[repr(u16)]
#[derive(Debug)]
pub enum WindowsEncodingID {
    Symbol = 0,
    UnicodeBMP = 1,
    ShiftJIS = 2,
    PRC = 3,
    Big5 = 4,
    Wansung = 5,
    Johab = 6,
    UnicodeFull = 10,
}

#[derive(Debug)]
pub enum EncodingID {
    Unicode(UnicodeEncodingID),
    Macintosh(MacintoshEncodingID),
    ISO(ISOEncodingID),
    Windows(WindowsEncodingID),
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
            PlatformID::Windows => {
                if self.encoding_id > 10 {
                    panic!("Unknown Windows Encoding ID: {}", self.encoding_id);
                }

                EncodingID::Windows(unsafe { std::mem::transmute(self.encoding_id) })
            }
            PlatformID::Custom => EncodingID::Custom(self.encoding_id),
            _ => panic!(
                "Unsupported Platform ID for Encoding ID retrieval: {:?}",
                self.platform_id()
            ),
        }
    }
}

pub trait CMAPSubtableTrait {
    /// Parses the CMAP subtable from the given data slice.
    /// data: The byte slice containing the CMAP subtable data (including header).
    fn parse(data: &[u8], encoding: &CMAPEncodingRecord) -> Self
    where
        Self: Sized;

    /// Maps a character code to a glyph index.
    fn char_to_glyph_index(&self, char_code: u32) -> Option<uint16>;
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

impl CMAPSubtableTrait for CMAPSubtable0 {
    fn parse(data: &[u8], encoding: &CMAPEncodingRecord) -> Self
    where
        Self: Sized,
    {
        let length = uint16::from_data(&data[2..]);

        let language = interpret_language(encoding.platform_id, uint16::from_data(&data[4..]));

        let mut glyph_id_array = [0u8; 256];
        glyph_id_array.copy_from_slice(&data[6..262]);

        CMAPSubtable0 {
            length,
            language,
            glyph_id_array,
        }
    }

    fn char_to_glyph_index(&self, char_code: u32) -> Option<uint16> {
        if char_code > 255 {
            return None;
        }

        Some(self.glyph_id_array[char_code as usize] as uint16)
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

    _seg_count: uint16,
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
            .field(
                "end_code",
                &self.end_code.iter().take(5).collect::<Vec<&uint16>>(),
            )
            .field(
                "start_code",
                &self.start_code.iter().take(5).collect::<Vec<&uint16>>(),
            )
            .field(
                "id_delta",
                &self.id_delta.iter().take(5).collect::<Vec<&int16>>(),
            )
            .field(
                "id_range_offset",
                &self
                    .id_range_offset
                    .iter()
                    .take(5)
                    .collect::<Vec<&uint16>>(),
            )
            .field("glyph_id_array_length", &self.glyph_id_array.len())
            .finish()
    }
}

impl CMAPSubtableTrait for CMAPSubtable4 {
    fn parse(data: &[u8], encoding: &CMAPEncodingRecord) -> Self
    where
        Self: Sized,
    {
        let length = uint16::from_data(&data[2..]);

        let language = interpret_language(encoding.platform_id, uint16::from_data(&data[4..]));

        let seg_count_x2 = uint16::from_data(&data[6..]);
        let search_range = uint16::from_data(&data[8..]);
        let entry_selector = uint16::from_data(&data[10..]);
        let range_shift = uint16::from_data(&data[12..]);

        let seg_count = seg_count_x2 / 2;

        let mut end_code = Vec::with_capacity(seg_count as usize);
        let mut start_code = Vec::with_capacity(seg_count as usize);
        let mut id_delta = Vec::with_capacity(seg_count as usize);
        let mut id_range_offset = Vec::with_capacity(seg_count as usize);

        let mut offset = 14;

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

        let glyph_id_array_length = (length as usize) - (offset);
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
            _seg_count: seg_count,
        }
    }

    fn char_to_glyph_index(&self, char_code: u32) -> Option<uint16> {
        if char_code > 0xFFFF {
            return None;
        }

        let char_code_u16 = char_code as uint16;

        let segment_index = self
            .end_code
            .iter()
            .position(|&end_code| char_code_u16 <= end_code);

        if segment_index.is_none() {
            return None;
        }

        let seg_index = segment_index.unwrap();
        let start_code = self.start_code[seg_index];
        // let end_code = self.end_code[seg_index];
        let id_delta = self.id_delta[seg_index];
        let id_range_offset = self.id_range_offset[seg_index];

        if start_code > char_code_u16 {
            return None;
        }

        if id_range_offset == 0 {
            Some(char_code_u16.wrapping_add_signed(id_delta))
        } else {
            let glyph_index_idx = (id_range_offset / 2) + (char_code_u16 - start_code)
                - (self._seg_count - seg_index as uint16);

            if (glyph_index_idx as usize) >= self.glyph_id_array.len() {
                return None;
            }

            let glyph_index = self.glyph_id_array[glyph_index_idx as usize];
            if glyph_index == 0 {
                return None;
            }

            Some(glyph_index.wrapping_add_signed(id_delta))
        }
    }
}

#[derive(Clone)]
pub struct CMAPSubtable6 {
    /// This is the length in bytes of the subtable.
    length: uint16,

    language: uint16,

    /// First character code of subrange.
    first_code: uint16,

    /// Number of character codes in subrange.
    entry_count: uint16,

    /// Array of glyph index values for character codes in the range.
    glyph_id_array: Vec<uint16>,
}

impl Debug for CMAPSubtable6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CMAPSubtable6")
            .field("length", &self.length)
            .field("language", &self.language)
            .field("first_code", &self.first_code)
            .field("entry_count", &self.entry_count)
            .field("glyph_id_array_length", &self.glyph_id_array.len())
            .finish()
    }
}

impl CMAPSubtableTrait for CMAPSubtable6 {
    fn parse(data: &[u8], encoding: &CMAPEncodingRecord) -> Self
    where
        Self: Sized,
    {
        let length = uint16::from_data(&data[2..]);

        let language = interpret_language(encoding.platform_id, uint16::from_data(&data[4..]));

        let first_code = uint16::from_data(&data[6..]);
        let entry_count = uint16::from_data(&data[8..]);
        let mut glyph_id_array = Vec::with_capacity(entry_count as usize);

        let mut offset = 10;

        for _ in 0..entry_count {
            glyph_id_array.push(uint16::from_data(&data[offset..]));
            offset += 2;
        }

        CMAPSubtable6 {
            length,
            language,
            first_code,
            entry_count,
            glyph_id_array,
        }
    }

    fn char_to_glyph_index(&self, char_code: u32) -> Option<uint16> {
        if char_code < self.first_code as u32 {
            return None;
        }

        if char_code > (self.first_code as u32 + self.entry_count as u32 - 1) {
            return None;
        }

        let index = char_code as uint16 - self.first_code;

        if index >= self.entry_count {
            return None;
        }

        Some(self.glyph_id_array[index as usize])
    }
}

#[derive(Clone, Debug)]
pub enum CMAPSubtable {
    Format0(CMAPSubtable0),
    Format2,
    Format4(CMAPSubtable4),
    Format6(CMAPSubtable6),
    Format8,
    Format10,
    Format12,
    Format13,
    Format14,
}

impl CMAPSubtable {
    pub fn parse_from_format_int(
        format: uint16,
        data: &[u8],
        encoding: &CMAPEncodingRecord,
    ) -> CMAPSubtable {
        match format {
            0 => CMAPSubtable::Format0(CMAPSubtable0::parse(data, encoding)),
            4 => CMAPSubtable::Format4(CMAPSubtable4::parse(data, encoding)),
            6 => CMAPSubtable::Format6(CMAPSubtable6::parse(data, encoding)),
            _ => panic!("Unsupported CMAP subtable format: {}", format),
        }
    }
}

#[derive(Clone, Default)]
pub struct CMAPTable {
    /// Table version number (0).
    pub version: uint16,

    /// Number of encoding tables that follow.
    pub num_tables: uint16,

    pub encoding_records: Vec<CMAPEncodingRecord>,

    pub subtables: Vec<CMAPSubtable>,
}

impl TableTrait for CMAPTable {
    fn parse(data: &[u8], _ctx: Option<ParseContext>) -> CMAPTable {
        let version = uint16::from_data(&data[0..2]);
        let num_tables = uint16::from_data(&data[2..4]);

        let mut cmap_table = CMAPTable::new(version, num_tables);

        let mut offset = 4;

        for _ in 0..num_tables {
            let platform_id = uint16::from_data(&data[offset..]);
            let encoding_id = uint16::from_data(&data[offset + 2..]);
            let subtable_offset = Offset32::from_data(&data[offset + 4..]);

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

            let subtable =
                CMAPSubtable::parse_from_format_int(format, &data[subtable_start..], encoding);

            cmap_table.subtables.push(subtable);
        }

        cmap_table
    }

    fn construct(&mut self, _data: &[u8]) {
        panic!(
            "CMAPTable does not require construction - simply parse the data instead with CMAPTable::parse()"
        );
    }
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

impl Debug for CMAPTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CMAPTable")
            .field("version", &self.version)
            .field("num_tables", &self.num_tables)
            .field("encoding_records_count", &self.encoding_records.len())
            .field(
                "encoding_records_preview",
                &self
                    .encoding_records
                    .iter()
                    .take(5)
                    .collect::<Vec<&CMAPEncodingRecord>>(),
            )
            .field("subtables_count", &self.subtables.len())
            .field(
                "subtables_preview",
                &self
                    .subtables
                    .iter()
                    .take(5)
                    .collect::<Vec<&CMAPSubtable>>(),
            )
            .finish()
    }
}
