//! Exercises `parse` against synthetic sfnt files built by hand, byte by
//! byte, rather than shipping a real font binary as a fixture.

use font_metrics::ParseError;

/// Assembles a minimal sfnt file: a table directory followed by the given
/// tables' raw bytes, in the order given.
fn build_font(tables: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let num_tables = tables.len() as u16;
    let mut out = Vec::new();

    out.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // sfnt version 1.0
    out.extend_from_slice(&num_tables.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes()); // searchRange, unused by parser
    out.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
    out.extend_from_slice(&0u16.to_be_bytes()); // rangeShift

    let mut data_offset = 12 + tables.len() * 16;
    let mut records = Vec::new();
    let mut data = Vec::new();
    for (tag, bytes) in tables {
        records.extend_from_slice(*tag);
        records.extend_from_slice(&0u32.to_be_bytes()); // checkSum, not verified
        records.extend_from_slice(&(data_offset as u32).to_be_bytes());
        records.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        data.extend_from_slice(bytes);
        data_offset += bytes.len();
    }

    out.extend_from_slice(&records);
    out.extend_from_slice(&data);
    out
}

/// A 54-byte `head` table with `unitsPerEm` set at its fixed offset (18).
fn head_table(units_per_em: u16) -> Vec<u8> {
    let mut t = vec![0u8; 54];
    t[18..20].copy_from_slice(&units_per_em.to_be_bytes());
    t
}

/// A 36-byte `hhea` table with ascender/descender/lineGap set.
fn hhea_table(ascender: i16, descender: i16, line_gap: i16) -> Vec<u8> {
    let mut t = vec![0u8; 36];
    t[4..6].copy_from_slice(&ascender.to_be_bytes());
    t[6..8].copy_from_slice(&descender.to_be_bytes());
    t[8..10].copy_from_slice(&line_gap.to_be_bytes());
    t
}

/// An `OS/2` table at the given version, long enough to hold every field
/// that version defines, with the typo/win/cap/x-height fields populated.
fn os2_table(
    version: u16,
    typo_ascender: i16,
    typo_descender: i16,
    typo_line_gap: i16,
    win_ascent: u16,
    win_descent: u16,
    x_height: i16,
    cap_height: i16,
) -> Vec<u8> {
    let len = if version >= 2 { 96 } else { 78 };
    let mut t = vec![0u8; len];
    t[0..2].copy_from_slice(&version.to_be_bytes());
    t[68..70].copy_from_slice(&typo_ascender.to_be_bytes());
    t[70..72].copy_from_slice(&typo_descender.to_be_bytes());
    t[72..74].copy_from_slice(&typo_line_gap.to_be_bytes());
    t[74..76].copy_from_slice(&win_ascent.to_be_bytes());
    t[76..78].copy_from_slice(&win_descent.to_be_bytes());
    if version >= 2 {
        t[86..88].copy_from_slice(&x_height.to_be_bytes());
        t[88..90].copy_from_slice(&cap_height.to_be_bytes());
    }
    t
}

#[test]
fn reads_head_and_hhea_metrics() {
    let head = head_table(2048);
    let hhea = hhea_table(1900, -500, 100);
    let font = build_font(&[(b"head", &head), (b"hhea", &hhea)]);

    let metrics = font_metrics::parse(&font).expect("valid synthetic font");
    assert_eq!(metrics.units_per_em, 2048);
    assert_eq!(metrics.ascender, 1900);
    assert_eq!(metrics.descender, -500);
    assert_eq!(metrics.line_gap, 100);
    assert_eq!(metrics.typo_ascender, None);
    assert_eq!(metrics.win_ascent, None);
    assert_eq!(metrics.cap_height, None);
}

#[test]
fn reads_os2_v4_metrics_including_cap_and_x_height() {
    let head = head_table(1000);
    let hhea = hhea_table(1900, -500, 0);
    let os2 = os2_table(4, 1984, -494, 0, 2728, 680, 1096, 1466);
    let font = build_font(&[(b"head", &head), (b"hhea", &hhea), (b"OS/2", &os2)]);

    let metrics = font_metrics::parse(&font).expect("valid synthetic font");
    assert_eq!(metrics.typo_ascender, Some(1984));
    assert_eq!(metrics.typo_descender, Some(-494));
    assert_eq!(metrics.win_ascent, Some(2728));
    assert_eq!(metrics.win_descent, Some(680));
    assert_eq!(metrics.x_height, Some(1096));
    assert_eq!(metrics.cap_height, Some(1466));
}

#[test]
fn os2_version_0_has_no_cap_or_x_height() {
    let head = head_table(1000);
    let hhea = hhea_table(1900, -500, 0);
    let os2 = os2_table(0, 1984, -494, 0, 2728, 680, 0, 0);
    let font = build_font(&[(b"head", &head), (b"hhea", &hhea), (b"OS/2", &os2)]);

    let metrics = font_metrics::parse(&font).expect("valid synthetic font");
    assert_eq!(metrics.win_ascent, Some(2728));
    assert_eq!(metrics.cap_height, None);
    assert_eq!(metrics.x_height, None);
}

#[test]
fn table_order_in_the_directory_does_not_matter() {
    let head = head_table(2048);
    let hhea = hhea_table(1900, -500, 100);
    // hhea listed before head, unlike every real font.
    let font = build_font(&[(b"hhea", &hhea), (b"head", &head)]);

    let metrics = font_metrics::parse(&font).expect("valid synthetic font");
    assert_eq!(metrics.units_per_em, 2048);
    assert_eq!(metrics.ascender, 1900);
}

#[test]
fn missing_hhea_is_an_error() {
    let head = head_table(2048);
    let font = build_font(&[(b"head", &head)]);
    assert_eq!(
        font_metrics::parse(&font),
        Err(ParseError::MissingTable("hhea"))
    );
}

#[test]
fn missing_head_is_an_error() {
    let hhea = hhea_table(1900, -500, 0);
    let font = build_font(&[(b"hhea", &hhea)]);
    assert_eq!(
        font_metrics::parse(&font),
        Err(ParseError::MissingTable("head"))
    );
}

#[test]
fn empty_file_is_too_short() {
    assert_eq!(font_metrics::parse(&[]), Err(ParseError::TooShort));
}

#[test]
fn garbage_sfnt_version_is_rejected() {
    let mut font = build_font(&[]);
    font[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_be_bytes());
    assert_eq!(font_metrics::parse(&font), Err(ParseError::NotASfntFile));
}

#[test]
fn true_type_collections_are_reported_as_unsupported() {
    let mut font = build_font(&[]);
    font[0..4].copy_from_slice(b"ttcf");
    assert_eq!(
        font_metrics::parse(&font),
        Err(ParseError::UnsupportedCollection)
    );
}
