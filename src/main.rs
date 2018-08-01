#![feature(rust_2018_preview)]
#![warn(rust_2018_idioms)]

use std::fs::File;
use std::io::{self, prelude::*, BufRead, BufReader};

use clap::ArgMatches;
use rayon::slice::ParallelSliceMut;

mod args;

const STDIN_FILENAME: &str = "-";

fn read_file(file: impl Read) -> Vec<String> {
    let buf_reader = BufReader::new(file);
    buf_reader.lines().filter_map(|l| l.ok()).collect()
}

fn write_result(lines: &[String]) -> io::Result<()> {
    let out = io::stdout();
    let mut handle = out.lock();

    for line in lines {
        handle.write(line.as_bytes())?;
        handle.write(b"\n")?;
    }

    Ok(())
}

fn sort(lines: &mut [String], matches: &'a ArgMatches<'a>) {
    if matches.is_present("fold") {
        // FIXME: Why doesn't this output the same as the sort built in?
        use caseless::default_case_fold_str as fold;

        lines.par_sort_unstable_by_key(|k| fold(&k));
        return;
    }

    lines.par_sort_unstable();
}

fn run_sort(matches: &'a ArgMatches<'a>) -> io::Result<()> {
    let files = matches
        .values_of("FILE")
        .unwrap_or(clap::Values::default())
        .collect::<Vec<&str>>();

    // FIXME: Be smarter with allocation.
    let mut lines = vec![];

    // No files were supplied so read from standard input.
    if files.is_empty() {
        let stdin = io::stdin();
        lines.extend(read_file(stdin.lock()));
    }

    for file in &files {
        // FIXME: Can we clean up this duplication from above?
        if file == &STDIN_FILENAME {
            let stdin = io::stdin();
            lines.extend(read_file(stdin.lock()));
            continue;
        }
        lines.extend(read_file(File::open(file)?));
    }

    sort(&mut lines, matches);

    write_result(&lines)
}

fn main() {
    let matches = args::matches();

    // FIXME: Better error messages.
    if let Err(e) = run_sort(&matches) {
        eprintln!("error: {}", e);
    }
}
