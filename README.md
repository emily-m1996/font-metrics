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

A TrueType collection (`.ttc`) bundles several fonts in one file. `parse`
reads the first one; `parse_at(&data, index)` reads a specific one, and
`font_count(&data)` reports how many there are (1 for a plain `.ttf`/`.otf`).

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

Pass `--json` (anywhere on the command line) to get one JSON object on
stdout instead. Fields missing from the font are `null`. The same output is
available from the library as `FontMetrics::to_json`.

```
$ cargo run --bin fontmetrics -- --json Inter-Regular.ttf
{"units_per_em":2048,"hhea_ascender":2728,"hhea_descender":-680,...}
```

## What's supported

Reads the sfnt table directory and pulls values out of `head`, `hhea`, and
`OS/2`. Works on both TrueType (`glyf`-based) and CFF-flavored OpenType
(`OTTO`) files, since the metrics tables are laid out the same way in
both, and on TrueType collections (`.ttc`), which bundle several of these
per file behind a shared header.

```
$ cargo run --bin fontmetrics -- NotoSansCJK.ttc 1
font 1 of 4 in this collection
units per em         1000
...
```

## License

MIT, see LICENSE.
