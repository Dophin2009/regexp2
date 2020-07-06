use crate::RegExp;

macro_rules! run_tests {
    ($exprs:expr, $valids:expr, $invalids:expr) => {{
        $exprs.iter().for_each(|&expr| {
            let re = RegExp::new_with_nfa(expr).unwrap();
            $valids.iter().for_each(|s| {
                assert!(
                    re.is_exact_match(s),
                    r#""{}" failed to match "{}""#,
                    expr,
                    s
                )
            });
            $invalids.iter().for_each(|s| {
                assert_eq!(re.is_exact_match(s), false, r#""{}" matched "{}""#, expr, s)
            });
        })
    }};
}

macro_rules! run_invalid_tests {
    ($exprs:expr) => {{
        $exprs.iter().for_each(|&expr| {
            RegExp::new_with_nfa(expr).unwrap_err();
        });
    }};
}

#[test]
fn test_blank() {
    let exprs = ["", "()", "(())", "((()))", "()()"];
    let valids = [""];
    let invalids = [" ", "a", "  "];
    run_tests!(&exprs, &valids, &invalids);
}

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

#[test]
fn test_alternate() {
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
}

#[test]
fn test_char_class() {
    let exprs = ["[abc]"];
    let valids = ["a", "b", "c"];
    let invalids = ["", "d", "ab", "bc", "ac"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[a-c]"];
    let valids = ["a", "b", "c"];
    let invalids = ["", "d", "ab", "bc", "ac"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[a-bd-e]"];
    let valids = ["a", "b", "d", "e"];
    let invalids = ["", "c", "f", "ab", "bc", "ac"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[a-bd-e]*"];
    let valids = ["", "a", "b", "d", "e", "aa", "ba", "ae", "abde", "eabd"];
    let invalids = [" ", "c", "f", "z", "ac", "addc"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[ab]b"];
    let valids = ["ab", "bb"];
    let invalids = ["", " ", "a", "b", "aa", "cb"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[a]]"];
    let valids = ["a]"];
    let invalids = ["", " ", "a", "]", "[a]", "[a]]", "b]"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"[\]-b]"];
    let valids = ["]", "b", "a", "^", "`", "_"];
    let invalids = ["", " ", "c", r"\", "[", "-", "]b", "]-b"];
    run_tests!(&exprs, &valids, &invalids);
}

#[test]
fn test_negation_symbol() {
    let exprs = ["[^B-D]"];
    let valids = ["a", "b", "c", "A", "E", "-", "1", "^", " "];
    let invalids = ["", "B", "C", "D"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[^1-8]"];
    let valids = ["0", "9", "a", "b", "A", "^", "[", "]"];
    let invalids = ["", "1", "3", "4", "8"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[a^1-8]"];
    let valids = ["a", "1", "3", "7", "8", "^"];
    let invalids = ["", "b", "0", "9"];
    run_tests!(&exprs, &valids, &invalids);
}

#[test]
fn test_wildcard() {
    let exprs = ["."];
    let valids = ["0", "9", "a", "b", "A", "^", "[", "何"];
    let invalids = ["", "\n"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = ["[.]", r"[\.]", r"\."];
    let valids = ["."];
    let invalids = ["", "a", "b", "4", "[", "\n", ".."];
    run_tests!(&exprs, &valids, &invalids);
}

#[test]
fn test_special_classes() {
    let exprs = [r"\d"];
    let valids = ["0", "1", "5", "9", "４", "８"];
    let invalids = ["", " ", "a", "b", "A", "-"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"[\d-]"];
    let valids = ["0", "1", "5", "9", "４", "８", "-"];
    let invalids = ["", " ", "a", "b", "A", "]", "["];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"[-\dA-Z]"];
    let valids = ["0", "1", "5", "9", "４", "８", "-", "A", "D", "Z"];
    let invalids = ["", " ", "a", "b", "]", "["];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\D"];
    let valids = [" ", "a", "b", "A", "-"];
    let invalids = ["", "0", "1", "5", "9", "４", "８"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\w"];
    let valids = ["a", "b", "A", "Q", "5", "9", "0", "_"];
    let invalids = ["", " ", "-", r"\", "["];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\W"];
    let valids = [" ", "-", r"\", "["];
    let invalids = ["", "a", "b", "A", "Q", "5", "9", "0", "_"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\n"];
    let valids = ["\n"];
    let invalids = ["", " ", "a", "A", "5", "_", "\t"];
    run_tests!(&exprs, &valids, &invalids);

    let exprs = [r"\s"];
    let valids = [
        " ", "\u{000c}", "\n", "\r", "\t", "\u{000b}", "\u{00a0}", "\u{1680}", "\u{2028}",
        "\u{2029}", "\u{202f}", "\u{205f}", "\u{3000}", "\u{feff}",
    ];
    let invalids = ["", "a", "A", "5", "_"];
    run_tests!(&exprs, &valids, &invalids);
}

#[test]
fn test_malformed() {
    let exprs = [
        "(", ")", "a(", "(()", "*", "|", "*a", "**", "a|", "a)*", "(ab",
    ];
    run_invalid_tests!(&exprs);
}
