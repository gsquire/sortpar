#![feature(rust_2018_preview)]
#![warn(rust_2018_idioms)]

use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, prelude::*, BufRead, BufReader};

use clap::ArgMatches;
use lazy_static::lazy_static;
use rayon::slice::ParallelSliceMut;
use regex::Regex;

mod args;

const STDIN_FILENAME: &str = "-";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Filter {
    LeadingBlanks,
    Dictionary,
    Fold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum SortType {
    GeneralNumeric,
    Regular,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct SortedFloat(f64);

impl Eq for SortedFloat {}

// Consider all errors to be equal.
impl Ord for SortedFloat {
    fn cmp(&self, other: &SortedFloat) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

// FIXME: Does it make sense to have this as a separate type?
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum SortResult {
    GeneralNumeric(SortedFloat),
    Regular(String),
}

fn apply_filter(input: &str, filter: Filter) -> String {
    use self::Filter::*;

    match filter {
        LeadingBlanks => leading_blanks_filter(input),
        Dictionary => dictionary_order_filter(input),
        Fold => fold_filter(input),
    }
}

// Based on the argument type given, find out how the lines should be sorted.
fn get_sort_type(matches: &'a ArgMatches<'a>) -> SortType {
    use self::SortType::*;

    if matches.is_present("general_numeric") {
        return GeneralNumeric;
    }

    Regular
}

// According to the GNU man page, if a line fails to parse or does not start with a number it is
// considered equal.
fn general_numeric_sort(input: &str) -> f64 {
    input.parse::<f64>().unwrap_or(0.0)
}

fn apply_sort_type(input: &str, sort_type: SortType) -> SortResult {
    use self::SortType::*;

    match sort_type {
        GeneralNumeric => SortResult::GeneralNumeric(SortedFloat(general_numeric_sort(input))),
        Regular => SortResult::Regular(input.to_string()),
    }
}

// Determine whether we need to transform the input to use in our sort comparator.
fn key_filter_function(input: &str, filters: &[Filter], matches: &'a ArgMatches<'a>) -> SortResult {
    if filters.is_empty() {
        return apply_sort_type(input, get_sort_type(matches));
    }

    let mut cmp = apply_filter(input, filters[0]);
    for filter in filters.into_iter().skip(1) {
        cmp = apply_filter(&cmp, *filter);
    }

    apply_sort_type(&cmp, get_sort_type(matches))
}

fn leading_blanks_filter(input: &str) -> String {
    input.trim_left().to_owned()
}

fn dictionary_order_filter(input: &str) -> String {
    lazy_static! {
        // It is safe to unwrap as we know this pattern compiles.
        static ref RE: Regex = Regex::new("[^[[:alnum:]][[:space:]]]").unwrap();
    }

    RE.replace_all(input, "").into_owned()
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

    // These filters can be added to any sorting style so check them first.
    if matches.is_present("leading_blanks") {
        filters.push(Filter::LeadingBlanks);
    }

    if matches.is_present("dictionary_order") {
        filters.push(Filter::Dictionary);
    }

    if matches.is_present("fold") {
        filters.push(Filter::Fold);
    }

    if matches.is_present("stable") {
        lines.par_sort_by_key(|k| key_filter_function(k, &filters, matches));
    } else {
        lines.par_sort_unstable_by_key(|k| key_filter_function(k, &filters, matches));
    }
}

// FIXME: We can probably do better with allocations here.
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
        } else {
            lines.extend(read_file(File::open(file)?));
        }
    }

    sort(&mut lines, matches);

    write_result(&lines)
}

fn main() {
    let matches = args::matches();

    // FIXME: Better error messages.
    if let Err(e) = run_sort(&matches) {
        eprintln!("error running sortpar: {}", e);
    }
}
