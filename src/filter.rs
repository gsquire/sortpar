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
pub(crate) fn filter_function<'s>(input: &'s str, filters: &[Filter]) -> Cow<'s, str> {
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
    use caseless::default_case_fold_str as fold;
    fold(input)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{dictionary_order_filter, filter_function, Filter};

    #[test]
    fn test_dictionary_filter() {
        let spaces = " ".repeat(7);
        let cases = vec![
            "simple",
            "number 7",
            "some other characters $%",
            "",
            &spaces,
        ];
        let expected = vec!["simple", "number 7", "some other characters ", "", &spaces];

        for case in cases.into_iter().zip(expected) {
            let actual = dictionary_order_filter(case.0);
            assert_eq!(actual, case.1);
        }
    }

    #[test]
    fn test_no_filter_returns_borrowed() {
        let input = "input";
        let result = filter_function(input, &[]);
        assert_eq!(result, Cow::Borrowed(input));
    }

    #[test]
    fn test_filter_returns_owned() {
        let input = " input";
        let expected = String::from("input");
        let result = filter_function(input, &[Filter::LeadingBlanks]);
        assert_eq!(result, Cow::Owned(expected) as Cow<'_, str>);
    }

    #[test]
    fn test_multiple_filters() {
        let input = " INPUT$";
        let expected = String::from("input");
        let result = filter_function(
            input,
            &[Filter::LeadingBlanks, Filter::Dictionary, Filter::Fold],
        );
        assert_eq!(result, Cow::Owned(expected) as Cow<'_, str>);
    }
}
