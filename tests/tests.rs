use std::fs::{File, OpenOptions};
use std::process::Command;
use std::time::Instant;

const CAT_PATH: &str = "/usr/bin/cat";
const FIXTURES_PATH: &str = "tests/fixtures";

fn run_and_compare_with_cat(test_args: &[&str]) {
    let rat_res = Command::new("cargo")
        .args(["run", "-p", "rat", "--"])
        .args(test_args)
        .output()
        .expect("failed to execute process");

    let cat_res = Command::new(CAT_PATH)
        .args(test_args)
        .output()
        .expect("failed to execute process");

    assert_eq!(
        String::from_utf8_lossy(&rat_res.stdout),
        String::from_utf8_lossy(&cat_res.stdout)
    );
}

fn run_and_timing_with_cat(test_args: &[&str]) {
    let start = Instant::now();
    let cat_res = Command::new(CAT_PATH)
        .args(test_args)
        .output()
        .expect("failed to execute process");
    let duration = start.elapsed();
    println!("Cat: {:?}", duration);

    let start = Instant::now();
    let rat_res = Command::new("cargo")
        .args(["run", "-p", "rat", "--"])
        .args(test_args)
        .output()
        .expect("failed to execute process");
    let duration = start.elapsed();
    println!("Rat: {:?}", duration);

    assert_eq!(
        String::from_utf8_lossy(&rat_res.stdout),
        String::from_utf8_lossy(&cat_res.stdout)
    );
}

#[test]
fn test_simple() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt")]);
}

#[test]
fn test_show_all() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-A"]);
}

#[test]
fn test_line_number() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-n"]);
}

#[test]
fn test_number_nonblank() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-n", "-b"]);
}

#[test]
fn test_nonprinting() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-t"]);
}

#[test]
fn test_nonprinting_ends() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-e"]);
}

#[test]
fn test_squeeze_blank() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-s"]);
}

#[test]
fn test_show_ends() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-E"]);
}

#[test]
fn test_show_tabs() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-T"]);
}

#[test]
fn test_show_nonprintable() {
    run_and_compare_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "256.txt"), "-v"]);
}

/// Put input string into file
fn print_to_file(input: &str, file: &str) {
    let input_file = File::create(file).expect("Fail creating file");
    Command::new("printf")
        .arg(input)
        .stdout(input_file.try_clone().expect("Failed to clone file handle"))
        .status()
        .expect("Failed to execute command");
}

fn run_and_compare(expect: &str, test_args: &[&str]) {
    let rat_res = Command::new("cargo")
        .args(["run", "-p", "rat", "--"])
        .args(test_args)
        .output()
        .expect("failed to execute process");
    assert_eq!(String::from_utf8_lossy(&rat_res.stdout), expect);
}

// cat-E.sh
#[test]
fn test_end() {
    print_to_file("a\rb\r\nc\n\r\nd\r", "in");
    run_and_compare("a\rb^M$\nc$\n^M$\nd\r", &["-E", "in"]);

    print_to_file("1\r", "in2");
    print_to_file("\n2\r\n", "in2b");
    run_and_compare("1^M$\n2^M$\n", &["-E", "in2", "in2b"]);

    print_to_file("1\r", "in2");
    print_to_file("2\r\n", "in2b");
    run_and_compare("1\r2^M$\n", &["-E", "in2", "in2b"]);
}

// Check when  the input is the same as the output
#[test]
fn test_self() {
    // rat out >> out
    print_to_file("x", "out");
    print_to_file("x", "out1");
    let output = OpenOptions::new()
        .append(true)
        .create(true)
        .open("out")
        .expect("Error creating file");

    let status = Command::new("cargo")
        .args(["run", "-p", "rat", "--", "out"])
        .stdout(output)
        .status()
        .expect("should return 1");
    assert_eq!(status.code().unwrap(), 1);

    //rat out out1 >> out
    let output = OpenOptions::new()
        .append(true)
        .create(true)
        .open("out")
        .expect("Error creating file");
    let status = Command::new("cargo")
        .args(["run", "-p", "rat", "--", "out", "out1"])
        .stdout(output)
        .status()
        .expect("should return 1");
    assert_eq!(status.code().unwrap(), 1);

    print_to_file("x", "fx");
    print_to_file("y", "fy");
    print_to_file("xy", "fxy");
}

#[test]
fn test_performance() {
    // generate a large file
    Command::new("bash")
        .arg("-c")
        .arg(format!(
            "base64 /dev/urandom | head -c 100MB > {}/largefile.txt",
            FIXTURES_PATH
        ))
        .output()
        .expect("failed to execute process");

    println!("Performance test");
    println!("-----------------");
    println!("largefile.txt");
    run_and_timing_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "largefile.txt")]);
    // println!("-----------------");
    // println!("largefile.txt -n");
    // run_and_timing_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "largefile.txt"), "-n"]);
    // println!("-----------------");
    // println!("largefile.txt -A");
    // run_and_timing_with_cat(&[&format!("{}/{}", FIXTURES_PATH, "largefile.txt"), "-A"]);
}
