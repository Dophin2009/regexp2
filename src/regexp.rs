pub struct RegExp {
    expr: String,
}

impl RegExp {
    pub fn new_with_nfa(expr: &str) -> Self {
        RegExp {
            expr: expr.to_owned(),
        }
    }
}
