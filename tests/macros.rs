#[allow(unused_macros)]

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
