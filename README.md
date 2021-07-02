[![Build status](https://github.com/Dophin2009/regexp2/workflows/ci/badge.svg)](https://github.com/Dophin2009/regexp2/actions)
[![Crates.io](https://img.shields.io/crates/v/regexp2.svg)](https://crates.io/crates/regexp2)
[![Docs.rs](https://docs.rs/regexp2/badge.svg)](https://docs.rs/regexp2)

# RegExp2

A toy regular expressions implementation.

# Usage

The following operators are supported:

-   `*`         : the Kleene star
-   `+`         : the "plus" operator, where `a+` is equivalent to `aa*` or
                  `a*a`
-   `?`         : the optional operator
-   `|`         : the union operator
-   `(` and `)` : grouping
-   \\          : escaping meta-characters
-   `[abc]`     : character classes with character ranges `[A-Z0-9]`
-   `[^abc]`    : negation of character classes
-   `\n`        : newline
-   `\d`, `\D`  : all Unicode decimal number characters and all non-decimal
                  number characters, respectively
-   `\w`, `\W`  : all word characters (alphanumeric and `_`) and non-word
                  characters, respectively
-   `\s`, `\S`  : all whitespace and non-whitespace characters, respectively
-   `.`         : any character except newline (`\n`)

A fairly arbitrary usage example:

```rust
use regexp2::RegExp;

fn main() {
    // Any sequence of a's and b's ending in abb.
    let mut re = RegExp::new("(a|b)*abb").unwrap();
    assert!(re.is_match("abb"));
    assert!(re.is_match("aababb"));

    // Any sequence of characters that are not B, C, D, E, F or any lowercase
    // letter.
    re = RegExp::new("[^B-Fa-z]*").unwrap();
    assert!(re.is_match("AGAQR"));

    // Any sequence of at least one digit followed by nothing or an
    // alphanumeric or underscore.
    re = RegExp::new(r"\d+\w?").unwrap();
    assert!(re.is_match("3a"));
    assert!(re.is_match("08m"));
    assert!(re.is_match("999_"));
}
```
