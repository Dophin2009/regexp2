use regexp2::RegExp;

include!("macros.rs");

#[test]
fn test_concat() {
    let exprs = ["ab", "(ab)", "(a)b", "a(b)", "()ab", "a()b"];
    let valids = ["ab"];
    let invalids = ["", " ", "a", "b", "c", "ab ", " ab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a ", "(a) ", "a( )", "(a )"];
    let valids = ["a "];
    let invalids = ["a", " ", " a", "a  ", " a "];
    run_tests!(&exprs, &valids, &invalids);
}
