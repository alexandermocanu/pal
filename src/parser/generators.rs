use crate::parser::*;

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

pub fn alt<T: 'static>(mut allowed: impl Iterator<Item = Parser<T>>) -> Parser<T> {
    if let Some(next) = allowed.next() {
        return next.or(alt(allowed));
    }

    Parser::empty(ParseError::Unit)
}

pub fn list(allowed: impl Iterator<Item = char>) -> Parser<char> {
    alt(allowed.map(char))
}

pub fn between<T: 'static, I: 'static, O: 'static>(
    a: Parser<T>,
    b: Parser<I>,
    c: Parser<O>,
) -> Parser<I> {
    a.right(b).left(c)
}

pub fn whitespace() -> Parser<char> {
    list([' ', '\n', '\t', '\r'].into_iter())
}

pub fn strip<T: 'static>(p: Parser<T>) -> Parser<T> {
    between(whitespace().many(), p, whitespace().many())
}

pub fn lowercase() -> Parser<char> {
    list('a'..='z')
}

pub fn uppercase() -> Parser<char> {
    list('A'..='Z')
}

pub fn other() -> Parser<char> {
    list(['_'].into_iter())
}

pub fn letter() -> Parser<char> {
    lowercase().or(uppercase()).or(other())
}

pub fn digit() -> Parser<char> {
    list('0'..='9')
}

pub fn alphanum() -> Parser<char> {
    letter().or(digit())
}

pub fn identifier() -> Parser<String> {
    strip(letter().chain(alphanum().many())).map(|(x, xs)| once(x).chain(xs).collect())
}

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
