#[allow(unused_macros)]

macro_rules! run_tests {
    ($exprs:expr, $valids:expr, $invalids:expr) => {{
        $exprs.iter().for_each(|&expr| {
            let nfa_re = RegExp::new(expr).unwrap();
            let dfa_re = RegExp::new_with_dfa(expr).unwrap();
            $valids.iter().for_each(|s| {
                assert!(
                    nfa_re.is_exact_match(s),
                    r#""{}" failed to match "{}" using nfa"#,
                    expr,
                    s
                );

                assert!(
                    dfa_re.is_exact_match(s),
                    r#""{}" failed to match "{}" using dfa"#,
                    expr,
                    s
                );
            });
            $invalids.iter().for_each(|s| {
                assert_eq!(
                    nfa_re.is_exact_match(s),
                    false,
                    r#""{}" matched "{}" using nfa"#,
                    expr,
                    s
                );
                assert_eq!(
                    dfa_re.is_exact_match(s),
                    false,
                    r#""{}" matched "{}" using dfa"#,
                    expr,
                    s
                );
            });
        })
    }};
}
