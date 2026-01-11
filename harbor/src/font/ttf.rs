#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::cmap::CMAPSubtableTrait;
use crate::font::tables::glyf::{CompositeGlyphFlags, GlyphDataType, Point};
use crate::font::tables::os2::OS2Table;
use crate::font::tables::{
    ParseContext, TableTrait, cmap, cvt, fpgm, gasp, glyf, hdmx, head, hhea, hmtx, loca, maxp,
    meta, name, os2, post, prep,
};
use crate::render::{Segment, Vertex, VertexMaker};

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
    Glyf(glyf::GlyfTable),
    CVT(cvt::CVTable),
    FPGM(fpgm::FPGMTable),
    Prep(prep::PrepTable),
    GASP(gasp::GASPTable),
    Meta(meta::MetaTable),
    HDMX(hdmx::HdmxTable),
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
            TableRecordData::Glyf(glyf_table) => glyf_table.fmt(f),
            TableRecordData::CVT(cvt_table) => cvt_table.fmt(f),
            TableRecordData::FPGM(fpgm_table) => fpgm_table.fmt(f),
            TableRecordData::Prep(prep_table) => prep_table.fmt(f),
            TableRecordData::GASP(gasp_table) => gasp_table.fmt(f),
            TableRecordData::Meta(meta_table) => meta_table.fmt(f),
            TableRecordData::HDMX(hdmx_table) => hdmx_table.fmt(f),
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
            b"glyf" => TableRecordData::Glyf({
                let mut glyf_table = glyf::GlyfTable::default().with_locas(
                    table_dir
                        ._loca_offsets
                        .clone()
                        .expect("loca offsets not set in TableDirectory."),
                );

                glyf_table.construct(data);
                glyf_table
            }),
            b"cvt " => TableRecordData::CVT(cvt::CVTable::parse(data, None)),
            b"fpgm" => TableRecordData::FPGM(fpgm::FPGMTable::parse(data, None)),
            b"prep" => TableRecordData::Prep(prep::PrepTable::parse(data, None)),
            b"gasp" => TableRecordData::GASP(gasp::GASPTable::parse(data, None)),
            b"meta" => TableRecordData::Meta(meta::MetaTable::parse(data, None)),
            b"hdmx" => TableRecordData::HDMX(hdmx::HdmxTable::parse(
                data,
                Some(ParseContext::Hdmx(
                    table_dir
                        ._maxp_num_glyphs
                        .expect("Number of glyphs not set in TableDirectory.")
                        as uint16,
                )),
            )),
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
                &tag_as_str(&self.table_tag).unwrap_or(String::from("Invalid Tag")),
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

    pub fn data(&self) -> &TableRecordData {
        &self._data
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
            b"glyf" => Some(Box::new(|table_dir: &TableDirectory| {
                table_dir._loca_offsets.is_some()
            })),
            b"hdmx" => Some(Box::new(|table_dir: &TableDirectory| {
                table_dir._maxp_num_glyphs.is_some()
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
    _loca_offsets: Option<Vec<uint32>>,

    _deferred_parse_queue: Vec<(Tag, Offset32, uint32, Box<dyn Fn(&TableDirectory) -> bool>)>,
}

impl TableDirectory {
    pub fn complete(&self) -> ParsedTableDirectory {
        ParsedTableDirectory {
            sfnt_version: self.sfnt_version,
            num_tables: self.num_tables,
            search_range: self.search_range,
            entry_selector: self.entry_selector,
            range_shift: self.range_shift,
            table_records: self.table_records.clone(),
            _maxp_num_glyphs: self._maxp_num_glyphs,
            _hhea_num_h_metrics: self._hhea_num_h_metrics,
            _head_mac_style: self._head_mac_style,
            _head_index_to_loc_format: self._head_index_to_loc_format,
            _loca_offsets: self._loca_offsets.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ParsedTableDirectory {
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
    _loca_offsets: Option<Vec<uint32>>,
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
            _loca_offsets: None,
            _deferred_parse_queue: Vec::new(),
        }
    }

    pub fn has_table(&self, tag: &Tag) -> bool {
        self.get_table_record(tag).is_some()
    }

    pub fn get_table_record(&self, tag: &Tag) -> Option<&TableRecord> {
        self.table_records
            .iter()
            .find(|record| &record.table_tag == tag)
    }
}

impl ParsedTableDirectory {
    pub fn get_table_record(&self, tag: &Tag) -> Option<&TableRecord> {
        self.table_records
            .iter()
            .find(|record| &record.table_tag == tag)
    }

    pub fn cmap_lookup(&self, char_code: uint32) -> Option<uint16> {
        if let Some(cmap_record) = self.get_table_record(b"cmap") {
            if let TableRecordData::CMAP(cmap_table) = &cmap_record._data {
                return cmap_table.char_to_glyph_index(char_code);
            }
        }

        None
    }

    pub fn from_char_code<F, T>(&self, char_code: uint32, f: F) -> Option<T>
    where
        F: Fn(usize) -> Option<T>,
    {
        if let Some(glyph_index) = self.cmap_lookup(char_code) {
            return f(glyph_index as usize);
        }

        None
    }

    pub fn units_per_em(&self) -> uint16 {
        if let Some(head_record) = self.get_table_record(b"head") {
            if let TableRecordData::Head(head_table) = &head_record._data {
                return head_table.units_per_em;
            }
        }

        panic!("Head table not found.");
    }

    pub fn glyph_index(&self, char_code: uint32) -> Option<uint16> {
        if let Some(cmap_record) = self.get_table_record(b"cmap") {
            if let TableRecordData::CMAP(cmap_table) = &cmap_record._data {
                return cmap_table.char_to_glyph_index(char_code);
            }
        }

        None
    }

    pub fn advance_width(&self, glyph_index: usize) -> Option<uint16> {
        if let Some(hmtx_record) = self.get_table_record(b"hmtx") {
            if let TableRecordData::HMtx(hmtx_table) = &hmtx_record._data {
                return hmtx_table
                    .h_metrics
                    .get(glyph_index)
                    .map(|h_metric| h_metric.advance_width);
            }
        }

        None
    }

    pub fn raw_line_gap(&self) -> Option<int16> {
        if let Some(hhea_record) = self.get_table_record(b"hhea") {
            if let TableRecordData::HHea(hhea_table) = &hhea_record._data {
                return Some(hhea_table.line_gap);
            }
        }

        None
    }

    pub fn line_gap(&self) -> Option<int16> {
        if let Some(hhea_record) = self.get_table_record(b"hhea") {
            if let TableRecordData::HHea(hhea_table) = &hhea_record._data {
                return Some(hhea_table.line_gap + hhea_table.ascender - hhea_table.descender);
            }
        }

        None
    }

    pub fn y_max(&self, glyph_index: usize) -> Option<int16> {
        if let Some(glyf_record) = self.get_table_record(b"glyf") {
            if let TableRecordData::Glyf(glyf_table) = &glyf_record._data {
                if let Some(glyph) = glyf_table.glyphs.get(glyph_index) {
                    return Some(glyph.header.y_max);
                }
            }
        }

        None
    }

    pub fn y_min(&self, glyph_index: usize) -> Option<int16> {
        if let Some(glyf_record) = self.get_table_record(b"glyf") {
            if let TableRecordData::Glyf(glyf_table) = &glyf_record._data {
                if let Some(glyph) = glyf_table.glyphs.get(glyph_index) {
                    return Some(glyph.header.y_min);
                }
            }
        }

        None
    }

    pub fn line_height(&self) -> Option<int16> {
        if let Some(os2_record) = self.get_table_record(b"OS/2") {
            if let TableRecordData::OS2(os2_table) = &os2_record._data {
                let (fs_selection, asc, desc, gap) = match os2_table {
                    OS2Table::V5(v5) => (
                        v5.fs_selection,
                        v5.s_typo_ascender,
                        v5.s_typo_descender,
                        v5.s_typo_line_gap,
                    ),
                    OS2Table::V4(t) | OS2Table::V3(t) | OS2Table::V2(t) => (
                        t.fs_selection,
                        t.s_typo_ascender,
                        t.s_typo_descender,
                        t.s_typo_line_gap,
                    ),
                    _ => panic!(""),
                };

                if fs_selection & 0x80 != 0 {
                    return Some(asc - desc + gap);
                }
            }
        }

        if let Some(hhea_record) = self.get_table_record(b"hhea") {
            if let TableRecordData::HHea(hhea_table) = &hhea_record._data {
                return Some(hhea_table.ascender - hhea_table.descender + hhea_table.line_gap);
            }
        }

        None
    }

    pub fn ascent(&self) -> Option<int16> {
        if let Some(os2_record) = self.get_table_record(b"OS/2") {
            if let TableRecordData::OS2(os2_table) = &os2_record._data {
                let (fs_selection, asc) = match os2_table {
                    OS2Table::V5(v5) => (v5.fs_selection, v5.s_typo_ascender),
                    OS2Table::V4(t) | OS2Table::V3(t) | OS2Table::V2(t) => {
                        (t.fs_selection, t.s_typo_ascender)
                    }
                    _ => panic!(""),
                };

                if fs_selection & 0x80 != 0 {
                    return Some(asc);
                }
            }
        }

        if let Some(hhea_record) = self.get_table_record(b"hhea") {
            if let TableRecordData::HHea(hhea_table) = &hhea_record._data {
                return Some(hhea_table.ascender);
            }
        }

        None
    }

    pub fn advance_width_from_char_code(&self, char_code: uint32) -> Option<uint16> {
        if let Some(glyph_index) = self.glyph_index(char_code) {
            return self.advance_width(glyph_index as usize);
        }

        None
    }

    pub fn make_glyph_segments(&self, glyph_index: usize, precision: f32, out: &mut Vec<Segment>) {
        let glyf = match self.get_table_record(b"glyf").unwrap().data() {
            TableRecordData::Glyf(glyf_table) => glyf_table,
            _ => {
                panic!("Glyf table not found.");
            }
        };

        let glyph = glyf.glyphs.get(glyph_index).unwrap();

        let mut segments = Vec::<Segment>::new();

        match &glyph.data {
            GlyphDataType::Simple(simple) => {
                for contour in &simple.contours {
                    let mut segment_part = Vec::<Segment>::new();

                    // populate segments
                    let contour_points = contour.points.clone();

                    let mut prev = if contour_points[0].on_curve {
                        contour_points[0].clone()
                    } else if contour_points[contour_points.len() - 1].on_curve {
                        contour_points[contour_points.len() - 1].clone()
                    } else {
                        Point::midpoint(
                            &contour_points[0],
                            &contour_points[contour_points.len() - 1],
                        )
                    };

                    let mut i = contour_points.len() - 1;
                    while segment_part.len() < contour_points.len() {
                        let mut curr = contour_points[i % contour_points.len()].clone();
                        let mut next = contour_points[(i + 1) % contour_points.len()].clone();

                        if curr.on_curve && next.on_curve {
                            // Line segment
                            segment_part.push(Segment::Line(curr.clone(), next));
                            prev = curr;
                        } else if curr.on_curve && !next.on_curve {
                            // Quadratic Bezier segment
                            while !next.on_curve {
                                let after_next =
                                    contour_points[(i + 2) % contour_points.len()].clone();

                                let control_point = next.clone();
                                let end_point = if after_next.on_curve {
                                    after_next.clone()
                                } else {
                                    Point::midpoint(&next, &after_next)
                                };

                                segment_part.push(Segment::Quadratic(
                                    curr.clone(),
                                    control_point,
                                    end_point.clone(),
                                ));

                                curr = end_point;
                                next = after_next;
                                i += 1;
                            }
                            prev = curr;
                        } else {
                            // curr is off-curve
                            let control_point = curr.clone();
                            let end_point = if next.on_curve {
                                next.clone()
                            } else {
                                Point::midpoint(&curr, &next)
                            };

                            segment_part.push(Segment::Quadratic(
                                prev.clone(),
                                control_point,
                                end_point.clone(),
                            ));

                            prev = end_point;
                        }

                        i += 1;
                    }

                    segments.extend(segment_part);

                    // println!("Segments: {:#?}", segments);
                }
            }
            GlyphDataType::Composite(composite) => {
                for component in &composite.components {
                    let mut segment_part = Vec::<Segment>::new();
                    let component_glyph_index = component.glyph_index as usize;

                    if let Some(post_record) = self.get_table_record(b"post") {
                        if let TableRecordData::Post(post_table) = &post_record._data {
                            println!(
                                "Component Glyph Name: {}",
                                post_table.glyph_name(component_glyph_index as u16).unwrap()
                            );
                        }
                    }

                    let mut component_segments: Vec<Segment> = Vec::new();
                    self.make_glyph_segments(
                        component_glyph_index,
                        precision,
                        &mut component_segments,
                    );

                    if component.flags & CompositeGlyphFlags::ScaledComponentOffset != 0 {
                        let (x, y) = (component.arg1, component.arg2);

                        for segment in &mut segment_part {
                            segment.translate(x, y);
                        }

                        let transform = &component.transform;

                        for segment in component_segments {
                            segment_part.push(segment.transformed(transform.clone()));
                        }
                    } else {
                        let transform = &component.transform;

                        for segment in component_segments {
                            segment_part.push(segment.transformed(transform.clone()));
                        }

                        let (x, y) = (component.arg1, component.arg2);

                        for segment in &mut segment_part {
                            segment.translate(x, y);
                        }
                    }

                    segments.extend(segment_part);
                }
            }
        }

        out.extend(segments);
    }

    pub fn make_glyph_vertices(&self, glyph_index: usize, precision: f32, out: &mut Vec<Point>) {
        let mut segments: Vec<Segment> = Vec::new();
        self.make_glyph_segments(glyph_index, precision, &mut segments);

        for segment in &segments {
            segment.flatten(out, precision);
        }
    }

    pub fn make_glyph_vertices_from_char_code(
        &self,
        char_code: uint32,
        precision: f32,
        out: &mut Vec<Point>,
    ) {
        if let Some(glyph_index) = self.glyph_index(char_code) {
            self.make_glyph_vertices(glyph_index as usize, precision, out);
        }
    }

    pub fn make_glyph_array(&self, char_codes: &[uint32], precision: f32, out: &mut Vec<Point>) {
        for &char_code in char_codes {
            if let Some(glyph_index) = self.glyph_index(char_code) {
                self.make_glyph_vertices(glyph_index as usize, precision, out);
            }
        }
    }

    pub fn make_glyph_array_from_str(&self, text: &str, precision: f32, out: &mut Vec<Point>) {
        let char_codes: Vec<uint32> = text.chars().map(|c| c as uint32).collect();
        self.make_glyph_array(&char_codes, precision, out);
    }

    pub fn rasterize(
        &self,
        text: &str,
        scale: f32,
        precision: f32,
        origin: (f32, f32),
        window_size: (f32, f32),
    ) -> Vec<Vertex> {
        let mut vertices: Vec<Vertex> = Vec::new();

        let mut pen_x = origin.0;
        let pen_y = origin.1;

        for c in text.chars() {
            let char_code = c as uint32;

            if let Some(glyph_index) = self.glyph_index(char_code) {
                let mut glyph_points: Vec<Point> = Vec::new();
                self.make_glyph_vertices(glyph_index as usize, precision, &mut glyph_points);

                vertices.extend(glyph_points.iter().map(|point| {
                    Vertex::clipped_from_point(
                        point,
                        (pen_x, pen_y),
                        scale,
                        window_size,
                        [0.0, 0.0, 0.0],
                    )
                }));

                if let Some(aw) = self.advance_width(glyph_index as usize) {
                    pen_x += aw as f32 * scale;
                }
            }
        }

        vertices
    }
}

fn update_table_directory_with_record(table_directory: &mut TableDirectory) {
    let table_record = table_directory.table_records.last().unwrap();

    match &table_record.table_tag {
        b"maxp" => {
            if let TableRecordData::MaxP(maxp_table) = &table_record._data {
                table_directory._maxp_num_glyphs = Some(match maxp_table {
                    maxp::MaxPTable::V0_5(table_v0_5) => table_v0_5.num_glyphs as usize,
                    maxp::MaxPTable::V1_0(table_v1_0) => table_v1_0.num_glyphs as usize,
                });
            }
        }
        b"hhea" => {
            if let TableRecordData::HHea(hhea_table) = &table_record._data {
                table_directory._hhea_num_h_metrics = Some(hhea_table.number_of_h_metrics as usize);
            }
        }
        b"head" => {
            if let TableRecordData::Head(head_table) = &table_record._data {
                table_directory._head_mac_style = Some(head_table.mac_style);
                table_directory._head_index_to_loc_format = Some(head_table.index_to_loc_format);
            }
        }
        b"loca" => {
            if let TableRecordData::Loca(loca_table) = &table_record._data {
                table_directory._loca_offsets = Some(match loca_table {
                    loca::LocaTable::Short(offsets) => offsets
                        .iter()
                        .map(|&offset| (offset as uint32) * 2)
                        .collect(),
                    loca::LocaTable::Long(offsets) => offsets.clone(),
                    _ => panic!("Loca table not constructed yet."),
                });
            }
        }
        _ => {}
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

        update_table_directory_with_record(&mut table_directory);

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
                    update_table_directory_with_record(&mut table_directory);

                    _ = table_directory._deferred_parse_queue.remove(i);

                    break 'req_update;
                }
            }
        }

        record_offset += 16;
    }

    table_directory
}
