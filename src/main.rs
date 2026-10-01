use std::env;
use std::fmt::Display;
use std::fs;
use std::process::ExitCode;

const USAGE: &str = "usage: fontmetrics [--json] <path-to-font-file> [font-index]";

fn main() -> ExitCode {
    let mut json = false;
    let mut positional = Vec::new();
    for arg in env::args().skip(1) {
        if arg == "--json" {
            json = true;
        } else {
            positional.push(arg);
        }
    }
    let mut args = positional.into_iter();

    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let font_index: u32 = match args.next() {
        Some(raw) => match raw.parse() {
            Ok(i) => i,
            Err(_) => {
                eprintln!("font-index must be a non-negative integer, got {raw}");
                return ExitCode::FAILURE;
            }
        },
        None => 0,
    };

    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("could not read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    // In JSON mode stdout must stay a single parseable object, so the
    // collection note goes to stderr instead.
    if let Ok(count) = font_metrics::font_count(&data) {
        if count > 1 {
            if json {
                eprintln!("font {font_index} of {count} in this collection");
            } else {
                println!("font {font_index} of {count} in this collection");
            }
        }
    }

    let metrics = match font_metrics::parse_at(&data, font_index) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    if json {
        println!("{}", metrics.to_json());
        return ExitCode::SUCCESS;
    }

    println!("units per em         {}", metrics.units_per_em);
    println!("hhea ascender        {}", metrics.ascender);
    println!("hhea descender       {}", metrics.descender);
    println!("hhea line gap        {}", metrics.line_gap);
    print_optional("OS/2 typo ascender  ", metrics.typo_ascender);
    print_optional("OS/2 typo descender ", metrics.typo_descender);
    print_optional("OS/2 typo line gap  ", metrics.typo_line_gap);
    print_optional("OS/2 win ascent     ", metrics.win_ascent);
    print_optional("OS/2 win descent    ", metrics.win_descent);
    print_optional("OS/2 cap height     ", metrics.cap_height);
    print_optional("OS/2 x-height       ", metrics.x_height);

    ExitCode::SUCCESS
}

fn print_optional<T: Display>(label: &str, value: Option<T>) {
    match value {
        Some(v) => println!("{label} {v}"),
        None => println!("{label} (not present in this file)"),
    }
}
