use regexp2::RegExp;

include!("macros.rs");

// #[test]
// fn test_blank() {
// let exprs = ["", "()", "(())", "((()))", "()()"];
// let valids = [""];
// let invalids = [" ", "a", "  "];
// run_tests!(&exprs, &valids, &invalids);
// }

#[test]
fn test_single() {
    let exprs = [" ", "( )", "(( ))", "(() )"];
    let valids = [" "];
    let invalids = ["", "a", "  "];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a", "(a)"];
    let valids = ["a"];
    let invalids = ["", "b", "a ", " a", "aa"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["b", "(b)"];
    let valids = ["b"];
    let invalids = ["", "a", "a ", " a", "aa"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["\"", "(\")"];
    let valids = ["\""];
    let invalids = ["", "a", "\" ", " \"", "\"\""];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\*", r"(\*)"];
    let valids = ["*"];
    let invalids = ["", " ", "a", "**"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\(", r"(\()", r"()\("];
    let valids = ["("];
    let invalids = ["", " ", ")", "()"];
    run_tests!(&exprs, &valids, &invalids);
}
