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

#[test]
pub fn char_parser_parses() {
    assert_eq!(char('a').parse("abc"), Ok(('a', "bc".to_string())));

    assert_eq!(
        char('a').parse("bc"),
        Err(ParseError::CharacterMismatch {
            expected: Some('a'),
            found: Some('b')
        })
    );
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

    assert_eq!(
        alt(['a', 'b'].into_iter().map(char)).parse("cba"),
        Err(ParseError::CharacterMismatch {
            expected: Some('b'),
            found: Some('c')
        })
    );
}

#[test]
pub fn list_parser_parses() {}
