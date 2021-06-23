use regexp2::RegExp;

macro_rules! run_invalid_tests {
    ($exprs:expr) => {{
        $exprs.iter().for_each(|&expr| {
            RegExp::new(expr).unwrap_err();
            RegExp::new_with_dfa(expr).unwrap_err();
        });
    }};
}

#[test]
fn test_malformed() {
    let exprs = [
        "(", ")", "a(", "(()", "*", "|", "*a", "**", "a|", "a)*", "(ab",
    ];
    run_invalid_tests!(&exprs);
}
