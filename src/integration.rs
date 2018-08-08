use std::fs::File;
use std::process::Command;

use assert_cmd::prelude::*;
use slurp::read_all_to_string;
use tempfile::tempdir;

#[test]
fn test_specify_standard_in() {
    let input = "baseball\napple\ncar";
    let expected = b"apple\nbaseball\ncar\n";

    let out = Command::main_binary()
        .unwrap()
        .arg("-")
        .with_stdin()
        .buffer(input)
        .output()
        .unwrap();

    assert_eq!(out.stdout, expected);
}

#[test]
fn test_leading_blanks() {
    let expected = b"a\n    b\n    c\nd\n";

    let out = Command::main_binary()
        .unwrap()
        .arg("-b")
        .arg("test_files/leading_blanks.txt")
        .unwrap();

    assert_eq!(out.stdout, expected);
}

#[test]
fn test_unique() {
    let expected = String::from("at the beginning\ncool\ntest\nzebra\n");

    let out = Command::main_binary()
        .unwrap()
        .arg("-u")
        .arg("test_files/unique.txt")
        .unwrap();

    let actual = String::from_utf8(out.stdout).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_fold_and_output() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("output.txt");
    let file = File::create(path.clone()).unwrap();
    let expected = String::from("APPLE\napple\nbike\nBIKE\ncar\nCAR\n");

    let result = Command::main_binary()
        .unwrap()
        .arg("-f")
        .arg("-o")
        .arg(path.clone())
        .arg("test_files/fold.txt")
        .ok();

    assert!(result.is_ok());

    let actual = read_all_to_string(path).unwrap();
    assert_eq!(actual, expected);

    drop(file);
}
