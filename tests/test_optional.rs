use regexp2::RegExp;

include!("macros.rs");

#[test]
fn test_optional() {
    let exprs = ["a?", "(a?)", "(a)?", "((a)?)"];
    let valids = ["", "a"];
    let invalids = [" ", " a", "aa", "ab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["ab?", "(ab?)", "(a)b?", "a(b?)", "a(b)?", "(a)(b)?"];
    let valids = ["a", "ab"];
    let invalids = ["", "b", "aba", "abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a?b", "(a?b)", "(a?)b", "a?(b)", "(a?)(b)"];
    let valids = ["b", "ab"];
    let invalids = ["", "a", "aab", "abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["(ab)?", "((ab)?)", "((a)(b))?"];
    let valids = ["", "ab"];
    let invalids = [" ", "a", "b", "aab", "abb", "abab"];
    run_tests!(&exprs, &valids, &invalids);
}
