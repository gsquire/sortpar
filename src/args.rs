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
            Arg::with_name("fold")
                .short("f")
                .long("ignore-case")
                .help("fold casing while sorting"),
        ).get_matches()
}
