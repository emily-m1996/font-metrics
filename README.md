# font-metrics

Font files store more than one definition of "line height," and the
definitions don't agree with each other. A TrueType or OpenType file has an
ascender/descender/line-gap triple in its `hhea` table, a second
independent triple in the `OS/2` table's typo fields, and a third pair in
the same table meant for clipping on Windows. Browsers, word processors,
and design tools each pick a different one of these, which is why the same
font can end up with visibly different line spacing depending on where you
open it.

This is a small library, and a thin CLI on top of it, that reads those
tables directly out of a font file's bytes and reports what's actually
stored there instead of asking a rendering engine to make a choice for you.

## Library

```rust
let data = std::fs::read("Inter-Regular.ttf")?;
let metrics = font_metrics::parse(&data)?;

println!("units per em: {}", metrics.units_per_em);
println!("hhea ascender: {}", metrics.ascender);
if let Some(cap_height) = metrics.cap_height {
    println!("cap height: {}", cap_height);
}
```

`parse` takes the whole file's bytes and returns a `FontMetrics` struct.
Fields that come from `hhea` are always present; fields that come from the
optional `OS/2` table are `Option`s, since not every font has one (or has
an old enough version to be missing the newer fields).

## CLI

```
$ cargo run --bin fontmetrics -- Inter-Regular.ttf
units per em         2048
hhea ascender        2728
hhea descender       -680
hhea line gap        0
OS/2 typo ascender   1984
OS/2 typo descender  -494
OS/2 typo line gap   0
OS/2 win ascent      2728
OS/2 win descent     680
OS/2 cap height      1466
OS/2 x-height        1096
```

## What's supported

Reads the sfnt table directory and pulls values out of `head`, `hhea`, and
`OS/2`. Works on both TrueType (`glyf`-based) and CFF-flavored OpenType
(`OTTO`) files, since the metrics tables are laid out the same way in
both. TrueType collections (`.ttc`) aren't handled yet.

## License

MIT, see LICENSE.
