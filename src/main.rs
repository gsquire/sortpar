#![feature(rust_2018_preview)]
#![warn(rust_2018_idioms)]

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, BufRead, BufReader, BufWriter};

use clap::ArgMatches;
use itertools::Itertools;
use lazy_static::lazy_static;
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use version_compare::Version;

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
    // FIXME: This didn't work for disk size strings. Do we need to write another crate?
    Human,
    Regular,
    VersionSort,
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

fn apply_filter(input: &str, filter: Filter) -> String {
    use self::Filter::*;

    match filter {
        LeadingBlanks => leading_blanks_filter(input),
        Dictionary => dictionary_order_filter(input),
        Fold => fold_filter(input),
    }
}

// Based on the argument type given, find out how the lines should be sorted.
// We know that we can only use one at a time since they all conflict with one another via clap.
fn get_sort_type(matches: &'a ArgMatches<'a>) -> SortType {
    use self::SortType::*;

    if matches.is_present("general_numeric") {
        return GeneralNumeric;
    } else if matches.is_present("human_numeric") {
        return Human;
    } else if matches.is_present("version_sort") {
        return VersionSort;
    }

    Regular
}

// According to the GNU man page, if a line fails to parse or does not start with a number it is
// considered equal.
fn general_numeric_sort(input: &str) -> f64 {
    input.parse::<f64>().unwrap_or(0.0)
}

fn apply_sort_type(a: &str, b: &str, sort_type: SortType) -> Ordering {
    use self::SortType::*;

    match sort_type {
        GeneralNumeric => {
            SortedFloat(general_numeric_sort(a)).cmp(&SortedFloat(general_numeric_sort(b)))
        }
        Human => natord::compare(a, b),
        Regular => a.cmp(b),
        VersionSort => {
            // We try to parse both strings as versions, falling back to natural ordering if need
            // be.
            if let Some(ver_a) = Version::from(a) {
                if let Some(ver_b) = Version::from(b) {
                    if let Some(o) = ver_a.compare(&ver_b).ord() {
                        return o;
                    } else {
                        return natord::compare(a, b);
                    }
                } else {
                    return natord::compare(a, b);
                }
            }
            natord::compare(a, b)
        }
    }
}

// Determine whether we need to transform the input to use in our sort comparator.
// FIXME: Can we avoid the extra allocations here and in the filter functions below?
fn filter_function(input: &'s str, filters: &[Filter]) -> Cow<'s, str> {
    if filters.is_empty() {
        return Cow::Borrowed(input);
    }

    let mut cmp = apply_filter(input, filters[0]);
    for filter in filters.into_iter().skip(1) {
        cmp = apply_filter(&cmp, *filter);
    }

    Cow::Owned(cmp)
}

fn leading_blanks_filter(input: &str) -> String {
    input.trim_left().to_string()
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

fn write_lines(lines: &[String], mut out: impl Write) -> io::Result<()> {
    for line in lines {
        out.write_all(line.as_bytes())?;
        out.write_all(b"\n")?;
    }

    Ok(())
}

fn write_result(lines: &[String], matches: &'a ArgMatches<'a>) -> io::Result<()> {
    if let Some(f) = matches.value_of("output") {
        let file = OpenOptions::new().create(true).write(true).open(f)?;
        let out = BufWriter::new(file);
        return write_lines(lines, out);
    }

    let out = io::stdout();
    let handle = out.lock();
    write_lines(lines, handle)
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
        lines.par_sort_by(|a, b| {
            let filtered_a = filter_function(a, &filters);
            let filtered_b = filter_function(b, &filters);
            if matches.is_present("reverse") {
                return apply_sort_type(&filtered_b, &filtered_a, get_sort_type(matches));
            }
            apply_sort_type(&filtered_a, &filtered_b, get_sort_type(matches))
        });
    } else {
        lines.par_sort_unstable_by(|a, b| {
            let filtered_a = filter_function(a, &filters);
            let filtered_b = filter_function(b, &filters);
            if matches.is_present("reverse") {
                return apply_sort_type(&filtered_b, &filtered_a, get_sort_type(matches));
            }
            apply_sort_type(&filtered_a, &filtered_b, get_sort_type(matches))
        });
    }
}

fn run_sort(matches: &'a ArgMatches<'a>) -> io::Result<()> {
    const DEFAULT_BUFFER_CAPACITY: usize = 2048;

    let files = matches
        .values_of("FILE")
        .unwrap_or_default()
        .collect::<Vec<&str>>();

    let mut lines = Vec::with_capacity(DEFAULT_BUFFER_CAPACITY);

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

    if matches.is_present("unique") {
        let mut unique = lines.into_iter().unique().collect::<Vec<String>>();
        sort(&mut unique, matches);
        return write_result(&unique, matches);
    }

    sort(&mut lines, matches);
    write_result(&lines, matches)
}

fn main() {
    let matches = args::matches();

    // FIXME: Better error messages.
    if let Err(e) = run_sort(&matches) {
        eprintln!("error running sortpar: {}", e);
    }
}
