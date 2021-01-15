/// Tiny program that takes in a regular expression and a string. If the string does not match the
/// pattern, the program exits with a non-zero status code.
use regexp2::RegExp;
use std::env;
use std::process;

const HELP: &str = "regextest <regex> <string>";

fn main() {
    let mut args = env::args().skip(1);
    let expr = match args.next() {
        Some(s) => s,
        None => {
            println!("{}", HELP);
            process::exit(1);
        }
    };
    let string = match args.next() {
        Some(s) => s,
        None => {
            println!("{}", HELP);
            process::exit(1);
        }
    };

    let regexp = RegExp::new(&expr).expect("Invalid regular expression");

    let code = if regexp.is_match(&string) { 0 } else { 1 };

    process::exit(code);
}
