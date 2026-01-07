use crate::font::ttf::otf_dtypes::*;
use crate::font::ttf::{TTCHeader, TableDirectory, parse_table_directory};

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

pub fn parse_ttc(data: &[u8]) -> (TTCHeader, Vec<TableDirectory>) {
    let ttc_header = parse_ttc_header(data);
    let mut table_directories = Vec::with_capacity(ttc_header.num_fonts() as usize);

    for table_dir_offset in ttc_header.table_directory_offsets() {
        let table_directory = parse_table_directory(data, Some(*table_dir_offset as usize));
        table_directories.push(table_directory);
    }

    println!("Parsed TTC Header: {:?}", ttc_header);
    println!("Parsed Table Directories: {:#?}", table_directories);

    (ttc_header, table_directories)
}
