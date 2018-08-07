use std::process::Command;

use assert_cmd::prelude::*;

#[test]
fn test_specify_standard_in() {
    let input = "baseball\napple\ncar";
    let expected = "apple\nbaseball\ncar\n".as_bytes();

    let out = Command::main_binary()
        .unwrap()
        .arg("-")
        .with_stdin()
        .buffer(input)
        .output()
        .unwrap();

    assert_eq!(out.stdout, expected);
}
