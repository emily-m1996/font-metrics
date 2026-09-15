//! Reads the metrics tables embedded in TrueType and OpenType font files.
//!
//! A font file stores several different notions of "ascent" and "descent",
//! and they routinely disagree with each other. This crate reads them
//! straight out of the binary tables instead of trusting a rendering
//! engine's guess about which one to use.

use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    TooShort,
    NotASfntFile,
    UnsupportedCollection,
    MissingTable(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TooShort => write!(f, "file is too short to be a font"),
            ParseError::NotASfntFile => {
                write!(f, "not a TrueType/OpenType file (bad sfnt version)")
            }
            ParseError::UnsupportedCollection => {
                write!(f, "TrueType collections (.ttc) are not supported yet")
            }
            ParseError::MissingTable(tag) => write!(f, "font has no '{tag}' table"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Metrics pulled from the `head`, `hhea`, and `OS/2` tables.
///
/// `ascender`/`descender`/`line_gap` come from `hhea`. `typo_*` is a second,
/// independent answer to "how tall is a line" that lives in `OS/2` and is
/// what CSS `line-height: normal` is supposed to follow. `win_*` is a third
/// answer, meant for clipping on Windows. All three can differ in the same
/// file, which is the reason this tool exists.
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub typo_ascender: Option<i16>,
    pub typo_descender: Option<i16>,
    pub typo_line_gap: Option<i16>,
    pub win_ascent: Option<u16>,
    pub win_descent: Option<u16>,
    pub cap_height: Option<i16>,
    pub x_height: Option<i16>,
}

fn u16_at(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn i16_at(data: &[u8], offset: usize) -> Option<i16> {
    u16_at(data, offset).map(|v| v as i16)
}

fn u32_at(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset + 4)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Looks up a table's (offset, length) in the sfnt table directory by tag.
fn find_table(data: &[u8], num_tables: u16, tag: &[u8; 4]) -> Option<(usize, usize)> {
    for i in 0..num_tables as usize {
        let record = 12 + i * 16;
        let record_tag = data.get(record..record + 4)?;
        if record_tag == tag {
            let offset = u32_at(data, record + 8)? as usize;
            let length = u32_at(data, record + 12)? as usize;
            return Some((offset, length));
        }
    }
    None
}

/// Parses metrics out of the raw bytes of a `.ttf` or `.otf` file.
pub fn parse(data: &[u8]) -> Result<FontMetrics, ParseError> {
    if data.len() < 12 {
        return Err(ParseError::TooShort);
    }
    let sfnt_version = u32_at(data, 0).ok_or(ParseError::TooShort)?;
    if sfnt_version == 0x74746366 {
        // 'ttcf': a collection has its own header in front of the table
        // directory, which this parser doesn't walk yet.
        return Err(ParseError::UnsupportedCollection);
    }
    if sfnt_version != 0x0001_0000 && sfnt_version != 0x4F54_544F {
        return Err(ParseError::NotASfntFile);
    }
    let num_tables = u16_at(data, 4).ok_or(ParseError::TooShort)?;

    let (head_offset, _) =
        find_table(data, num_tables, b"head").ok_or(ParseError::MissingTable("head"))?;
    let units_per_em =
        u16_at(data, head_offset + 18).ok_or(ParseError::MissingTable("head"))?;

    let (hhea_offset, _) =
        find_table(data, num_tables, b"hhea").ok_or(ParseError::MissingTable("hhea"))?;
    let ascender = i16_at(data, hhea_offset + 4).ok_or(ParseError::MissingTable("hhea"))?;
    let descender = i16_at(data, hhea_offset + 6).ok_or(ParseError::MissingTable("hhea"))?;
    let line_gap = i16_at(data, hhea_offset + 8).ok_or(ParseError::MissingTable("hhea"))?;

    let mut metrics = FontMetrics {
        units_per_em,
        ascender,
        descender,
        line_gap,
        typo_ascender: None,
        typo_descender: None,
        typo_line_gap: None,
        win_ascent: None,
        win_descent: None,
        cap_height: None,
        x_height: None,
    };

    // OS/2 is optional in the spec, and its length varies by version, so
    // every field past the version byte is read defensively.
    if let Some((os2_offset, os2_len)) = find_table(data, num_tables, b"OS/2") {
        let version = u16_at(data, os2_offset).unwrap_or(0);
        if os2_len >= 78 {
            metrics.typo_ascender = i16_at(data, os2_offset + 68);
            metrics.typo_descender = i16_at(data, os2_offset + 70);
            metrics.typo_line_gap = i16_at(data, os2_offset + 72);
            metrics.win_ascent = u16_at(data, os2_offset + 74);
            metrics.win_descent = u16_at(data, os2_offset + 76);
        }
        if version >= 2 && os2_len >= 90 {
            metrics.x_height = i16_at(data, os2_offset + 86);
            metrics.cap_height = i16_at(data, os2_offset + 88);
        }
    }

    Ok(metrics)
}
