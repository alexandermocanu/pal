use crate::parser::*;

/// Matches exactly one [`char`].
pub fn char(allowed: char) -> Parser<char> {
    Parser::new(move |input| {
        let mut chars = input.chars();

        match chars.next() {
            Some(c) if c == allowed => Ok((c, chars.collect())),
            res => Err(ParseError::CharacterMismatch {
                expected: Some(allowed),
                found: res,
            }),
        }
    })
}

/// Turns an iterator of [`Parser<T>`] into a [`Parser<T>`] by applying `or` recursively.
pub fn alt<T: 'static>(mut allowed: impl Iterator<Item = Parser<T>>) -> Parser<T> {
    if let Some(next) = allowed.next() {
        return next.or(alt(allowed));
    }

    Parser::empty(ParseError::Unit)
}

/// Turns an iterator of [`char`] into a [`Parser<char>`] by applying `or` recursively.
pub fn list(allowed: impl Iterator<Item = char>) -> Parser<char> {
    alt(allowed.map(char))
}

/// Generates a [`Parser`] that expects 3 matches in a row, and drops the first and the last.
pub fn between<T: 'static, I: 'static, O: 'static>(
    a: Parser<T>,
    b: Parser<I>,
    c: Parser<O>,
) -> Parser<I> {
    a.right(b).left(c)
}

/// Generates a parser for whitespace characters.
pub fn whitespace() -> Parser<char> {
    list([' ', '\n', '\t', '\r'].into_iter())
}

/// Generates a parser that ignores whitespace characters.
pub fn strip<T: 'static>(p: Parser<T>) -> Parser<T> {
    between(whitespace().many(), p, whitespace().many())
}

/// Generates a parser that matches on all lowercase alphabetic characters.
pub fn lowercase() -> Parser<char> {
    list('a'..='z')
}

/// Generates a parser that matches on all uppercase alphabetic characters.
pub fn uppercase() -> Parser<char> {
    list('A'..='Z')
}

/// Generates a parser that matches on all non-alphabetic valid identifier characters.
pub fn other() -> Parser<char> {
    list(['_'].into_iter())
}

/// Generates a parser that matches on any letter (or non-alphabetic identifier character).
pub fn letter() -> Parser<char> {
    lowercase().or(uppercase()).or(other())
}

/// Generates a parser that matches on all numerical digits.
pub fn digit() -> Parser<char> {
    list('0'..='9')
}

/// Generates a parser that matches on all alphanumeric characters (or non-alphabetic identifier
/// characters).
pub fn alphanum() -> Parser<char> {
    letter().or(digit())
}

/// Generates a parser that matches on any possible identifier.
pub fn identifier() -> Parser<String> {
    strip(letter().chain(alphanum().many())).map(|(x, xs)| once(x).chain(xs).collect())
}

/// Generates a parser that matches on one exact given string-like item. This can be used to parse
/// for specific keywords like `if`, `while` and similar. Does not ignore whitespace.
pub fn string(input: impl ToString) -> Parser<String> {
    let input = input.to_string();
    let mut chars = input.chars();

    if let Some(next) = chars.next() {
        return char(next)
            .chain(string(chars.as_str()))
            .map(|(x, xs)| once(x).chain(xs.chars()).collect());
    }

    Parser::pure("".to_string())
}

/// Generates a parser that matches on one exact given string-like item. This can be used to parse
/// for specific keywords like `if`, `while` and similar. Ignores whitespace.
pub fn symbol(input: impl ToString) -> Parser<String> {
    strip(string(input))
}

#[test]
pub fn char_parser_parses() {
    assert_eq!(char('a').parse("abc"), Ok(('a', "bc".to_string())));

    assert!(char('a').parse("bc").is_err());
}

#[test]
pub fn alt_parser_parses() {
    assert_eq!(
        alt(['a', 'b'].into_iter().map(char)).parse("abc"),
        Ok(('a', "bc".to_string()))
    );

    assert_eq!(
        alt(['a', 'b'].into_iter().map(char)).parse("bac"),
        Ok(('b', "ac".to_string()))
    );

    assert!(alt(['a', 'b'].into_iter().map(char)).parse("cba").is_err(),);
}

#[test]
pub fn identifiers_parse() {
    assert_eq!(
        identifier().parse("abcdef123 fuck"),
        Ok(("abcdef123".to_string(), "fuck".to_string()))
    );

    assert!(identifier().parse("123abc").is_err(),)
}

#[test]
fn symbols_parse() {
    assert_eq!(
        symbol("fn").parse("fn hello"),
        Ok(("fn".to_string(), "hello".to_string()))
    );

    assert!(symbol("fn").parse("nf hello").is_err());
}
