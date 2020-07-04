pub struct RegExp {
    expr: String,
}

impl RegExp {
    pub fn new_nfa(expr: &str) -> Self {
        let regexp = RegExp {
            expr: expr.to_owned(),
        };
        regexp
    }
}
