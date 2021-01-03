# RegExp2

This is the second attempt at a regular expression parser and matcher. The code
`regexp2` is significantly less spaghetti and more flexible than that of
[`regexp`](../regexp). More operators and syntax are supported.

Currently, NFA and DFA backends is supported. DFAs are constructed by converting
from NFAs. A similar, but more generic parsing algorithm to that of `regexp` is
used, and the equivalent NFA of a regular expression is created using the subset
construction described in Algorithm 3.23 in *Compilers: Principles, Techniques,
and Tool, Second Edition*.

## Usage

The following operators are supported:

-   `*` : the Kleene star
-   `+` : the "plus" operator, where `a+` is equivalent to `aa*` or `a*a`
-   `?` : the optional operator
-   `|` : the union operator
-   `(` and `)` : grouping
-   \\ : escaping meta-characters
-   `[abc]`, `[a-z]`, `[A-Z0-9]` : character classes with character ranges
-   `[^abc]` : negation of character classes
-   `\d`, `\D` : all Unicode decimal number characters and all non-decimal
    number characters, respectively
-   `\w`, `\W` : all word characters (alphanumeric and `_`) and non-word
    characters, respectively
-   `\s`, `\S` : all whitespace (equivalent to
    `[ \u{000c}\n\r\t\u{000b}\u{00a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]`
    ) and non-whitespace characters, respectively
-   `.` : any character except newline (`\n`)

A fairly arbitrary usage example:

``` rust
use regexp2::RegExp;

fn main() {
  // Any sequence of a's and b's ending in abb.
  let mut re = RegExp::new("(a|b)*abb");
  assert!(re.is_match("abb"));
  assert!(re.is_match("aababb"));

  // Any sequence of characters that are not B, C, D, E, F or any lowercase letter.
  re = RegExp::new_with_dfa("[^B-Fa-z]*");
  assert!(re.is_match("AGAQR"));

  // Any sequence of at least one digit followed by nothing or an alphanumeric or underscore.
  re = RegExp::new_with_dfa("\d+\w?");
  assert!(re.is_match("3a"));
  assert!(re.is_match("08m"));
  assert!(re.is_match("999_"));
}
```
