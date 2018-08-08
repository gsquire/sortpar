#![feature(rust_2018_preview)]
#![warn(rust_2018_idioms)]

use std::fs::File;
use std::io::{self, prelude::*, BufRead, BufReader};

use clap::ArgMatches;
use rayon::slice::ParallelSliceMut;

mod args;

const STDIN_FILENAME: &str = "-";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Filter {
    LeadingBlanks,
    Fold,
}

fn apply_filter(input: &str, filter: &Filter) -> String {
    use self::Filter::*;

    match filter {
        LeadingBlanks => leading_blanks_filter(input),
        Fold => fold_filter(input),
    }
}

// Determine whether we need to transform the input to use in our sort comparator.
fn key_filter_function(input: &str, filters: &[Filter]) -> String {
    if filters.len() == 0 {
        return input.to_owned();
    }

    let mut cmp = apply_filter(input, &filters[0]);
    for filter in filters.iter().skip(1) {
        cmp = apply_filter(&cmp, filter);
    }

    cmp
}

fn leading_blanks_filter(input: &str) -> String {
    input.trim_left().to_owned()
}

fn fold_filter(input: &str) -> String {
    // FIXME: Why doesn't this output the same as the sort built in? Do we care?
    use caseless::default_case_fold_str as fold;
    fold(input)
}

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
    let mut filters = Vec::new();

    if matches.is_present("leading_blanks") {
        filters.push(Filter::LeadingBlanks);
    }

    if matches.is_present("fold") {
        filters.push(Filter::Fold);
    }

    lines.par_sort_unstable_by_key(|k| key_filter_function(k, &filters));
}

fn run_sort(matches: &'a ArgMatches<'a>) -> io::Result<()> {
    let files = matches
        .values_of("FILE")
        .unwrap_or(clap::Values::default())
        .collect::<Vec<&str>>();

    let mut lines = Vec::new();

    // No files were supplied so read from standard input.
    if files.is_empty() {
        lines.extend(read_file(io::stdin().lock()));
    }

    for file in &files {
        if file == &STDIN_FILENAME {
            lines.extend(read_file(io::stdin().lock()));
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
