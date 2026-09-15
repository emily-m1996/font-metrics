use std::env;
use std::fmt::Display;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: fontmetrics <path-to-font-file>");
            return ExitCode::FAILURE;
        }
    };

    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("could not read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let metrics = match font_metrics::parse(&data) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };

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
