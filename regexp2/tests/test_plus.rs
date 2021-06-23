use regexp2::RegExp;

include!("macros.rs");

#[test]
fn test_plus() {
    let exprs = ["a+", "(a+)", "(a)+", "((a)+)", "aa*", "a*a"];
    let valids = ["a", "aa", "aaa"];
    let invalids = ["", " ", " a", "ab", "aaaab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["ab+", "(ab+)", "(a)b+", "a(b+)", "a(b)+", "(a)(b)+", "abb*"];
    let valids = ["ab", "abb", "abbb"];
    let invalids = ["", "a", "b", "aba", " abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a+b", "(a+b)", "(a+)b", "a+(b)", "(a+)(b)", "aa*b", "a*ab"];
    let valids = ["ab", "aab", "aaab"];
    let invalids = ["", "a", "b", "abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["(ab)+", "((ab)+)", "((a)(b))+", "ab(ab)*"];
    let valids = ["ab", "abab", "ababab"];
    let invalids = ["", " ", "a", "b", "aab", "abb"];
    run_tests!(&exprs, &valids, &invalids);
}
