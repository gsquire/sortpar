use clap::{crate_authors, crate_version};
use clap::{App, Arg, ArgMatches};

pub(crate) fn matches<'a>() -> ArgMatches<'a> {
    App::new("sortpar")
        .about("sort in parallel")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("FILE")
                .value_name("FILE")
                .help("the list of files to sort")
                .multiple(true)
                .index(1),
        ).arg(
            Arg::with_name("leading_blanks")
                .short("b")
                .long("ignore-leading-blanks")
                .help("ignore leading blanks"),
        ).arg(
            Arg::with_name("dictionary_order")
                .short("d")
                .long("dictionary-order")
                .help("consider only blanks and alphanumeric characters"),
        ).arg(
            Arg::with_name("fold")
                .short("f")
                .long("ignore-case")
                .help("fold casing while sorting"),
        ).arg(
            Arg::with_name("general_numeric")
                .short("g")
                .long("general-numeric-sort")
                .help("compare according to general numerical value"),
        ).arg(
            Arg::with_name("stable")
                .short("s")
                .long("stable")
                .help("use stable sort"),
        ).get_matches()
}
