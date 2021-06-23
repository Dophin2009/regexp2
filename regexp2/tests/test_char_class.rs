use regexp2::RegExp;

include!("macros.rs");

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
