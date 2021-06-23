use regexp2::RegExp;

include!("macros.rs");

#[test]
fn test_union() {
    let exprs = ["a|b", "(a|b)", "(a)|b", "a|(b)", "((a)|b)"];
    let valids = ["a", "b"];
    let invalids = ["", " ", "c", "a ", " a", "ab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a|b|c", "(a|b)|c", "(a)|b|(c)", "a|(b)|c", "a|(b|c)"];
    let valids = ["a", "b", "c"];
    let invalids = ["", " ", "d", "a ", " a", "ab", "bc"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\*|a", r"\*|(a)"];
    let valids = ["*", "a"];
    let invalids = ["", " ", "*a", r"\*"];
    run_tests!(&exprs, &valids, &invalids);
}
