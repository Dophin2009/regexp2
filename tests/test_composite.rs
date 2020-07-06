use regexp2::RegExp;

include!("macros.rs");

#[test]
fn test_composite() {
    let exprs = ["(a|b)*", "((a)|b)*"];
    let valids = ["", "a", "b", "aa", "bb", "ab", "aabb", "abbb"];
    let invalids = [" ", "a ", " ab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["(a|bc)*", "(a|(bc))*", "((a)|(bc))*"];
    let valids = ["", "a", "bc", "abc", "bca", "aabc", "abcbc"];
    let invalids = [" ", "c", "b", "ab", "ac", "ba", "abcb", "abcc"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a|b*", "a|(b*)", "(a)|(b)*", "(a|b*)"];
    let valids = ["", "a", "b", "bb", "bbb"];
    let invalids = [" ", "aa", "ab", "ba", "aab", "aba"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["a*|b", "(a)*|b", "(a*)|b", "a*|(b)"];
    let valids = ["", "a", "b", "aa", "aaa"];
    let invalids = [" ", "ab", "aab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["(a|b)*abb", "(a|b)*(abb)", "(a|b)*a(bb)"];
    let valids = ["abb", "aabb", "babb", "aababb"];
    let invalids = ["", "ab", "aba", "bab"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["ab(a|b)*abb"];
    let valids = ["ababb", "abaabb", "abbabb", "abababb", "abbaabb"];
    let invalids = ["", "ab", "abab", "abb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\d+\w?"];
    let valids = ["1", "3a", "08m", "046b", "999_"];
    let invalids = ["", " ", "a", "55xy"];
    run_tests!(&exprs, &valids, &invalids);
}
