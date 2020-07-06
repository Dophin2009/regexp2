use regexp2::RegExp;

include!("macros.rs");

#[test]
fn test_kleene() {
    let exprs = ["a*", "(a*)", "(a)*", "((a)*)"];
    let valids = ["", "a", "aa", "aaa"];
    let invalids = [" ", " a", "ab", "aaaab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["ab*", "(ab*)", "(a)b*", "a(b*)", "a(b)*", "(a)(b)*"];
    let valids = ["a", "ab", "abb", "abbb"];
    let invalids = ["", "b", "aba", " abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a*b", "(a*b)", "(a*)b", "a*(b)", "(a*)(b)"];
    let valids = ["b", "ab", "aab", "aaab"];
    let invalids = ["", "a", "abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["(ab)*", "((ab)*)", "((a)(b))*"];
    let valids = ["", "ab", "abab"];
    let invalids = [" ", "a", "b", "aab", "abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["(ab*)*", "((a)b*)*", "((a)(b)*)*", "((ab*)*)"];
    let valids = ["", "a", "ab", "abb", "abab", "ababa", "abbabb"];
    let invalids = [" ", "b", "ba", "babb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\**", r"(\*)*", r"()\**"];
    let valids = ["", "*", "**", "***"];
    let invalids = [" ", "* ", " *", r"\*", r"\"];
    run_tests!(&exprs, &valids, &invalids);
}
