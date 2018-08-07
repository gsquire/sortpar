use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Filter {
    LeadingBlanks,
    Dictionary,
    Fold,
}

fn apply_filter(input: &str, filter: Filter) -> String {
    use self::Filter::*;

    match filter {
        LeadingBlanks => leading_blanks_filter(input),
        Dictionary => dictionary_order_filter(input),
        Fold => fold_filter(input),
    }
}

// Determine whether we need to transform the input to use in our sort comparator.
// FIXME: Can we avoid the extra allocations here and in the filter functions below?
pub(crate) fn filter_function(input: &'s str, filters: &[Filter]) -> Cow<'s, str> {
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
