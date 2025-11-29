use std::{iter::once, str::Chars, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    CharacterMismatch,
}

#[derive(Clone)]
pub struct Parser<T> {
    parser: Arc<dyn Fn(String) -> Result<(T, String), ParseError>>,
}

impl<T> Parser<T> {
    pub fn new(parser: impl Fn(String) -> Result<(T, String), ParseError> + 'static) -> Parser<T> {
        Parser {
            parser: Arc::new(parser),
        }
    }

    pub fn lazy(parser_factory: impl Fn() -> Parser<T> + 'static) -> Parser<T> {
        Parser::new(move |input| {
            let parser = parser_factory();
            parser.parse(input)
        })
    }

    pub fn map<O>(self, func: impl Fn(T) -> O + 'static) -> Parser<O>
    where
        T: 'static,
        O: 'static,
    {
        Parser::new(move |input| {
            self.parse(input)
                .map(|(result, remaining)| (func(result), remaining))
        })
    }

    pub fn pure(value: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        Parser::new(move |input| Ok((value.clone(), input)))
    }

    pub fn maybe(self) -> Parser<Option<T>>
    where
        T: 'static,
    {
        Parser::new(move |input| match self.parse(input.clone()) {
            Ok((result, input)) => Ok((Some(result), input)),
            Err(_) => Ok((None, input)),
        })
    }

    pub fn many(self) -> Parser<Vec<T>>
    where
        T: Clone + 'static,
    {
        self.clone()
            .and(Parser::lazy(move || self.clone().many().maybe()))
            .map(|(a, b)| Some(a).into_iter().chain(b.unwrap_or(vec![])).collect())
            .or(Parser::pure(vec![]))
    }

    pub fn some(self) -> Parser<Vec<T>>
    where
        T: Clone + 'static,
    {
        self.clone()
            .and(self.many())
            .map(|(a, b)| once(a).chain(b).collect())
    }

    pub fn and<O>(self, other: Parser<O>) -> Parser<(T, O)>
    where
        T: 'static,
        O: 'static,
    {
        Parser::new(move |input| {
            self.parse(input).and_then(|(result_a, input_a)| {
                other
                    .parse(input_a)
                    .map(|(result_b, input_b)| ((result_a, result_b), input_b))
            })
        })
    }

    pub fn left<O>(self, other: Parser<O>) -> Parser<T>
    where
        T: 'static,
        O: 'static,
    {
        self.and(other).map(|(value, _)| value)
    }

    pub fn right<O>(self, other: Parser<O>) -> Parser<O>
    where
        T: 'static,
        O: 'static,
    {
        self.and(other).map(|(_, value)| value)
    }

    pub fn empty(value: ParseError) -> Parser<T> {
        Parser::new(move |_| Err(value))
    }

    pub fn or(self, other: Parser<T>) -> Parser<T>
    where
        T: 'static,
    {
        Parser::new(move |input| self.parse(input.clone()).or_else(|_| other.parse(input)))
    }

    pub fn between<O, U>(self, b: Parser<O>, c: Parser<U>) -> Parser<T>
    where
        T: 'static,
        O: 'static,
        U: 'static,
    {
        b.right(self).left(c)
    }

    pub fn otherwise(self, other: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        self.or(Parser::pure(other))
    }

    pub fn strip(self) -> Parser<T>
    where
        T: 'static,
    {
        self.between(whitespace().many(), whitespace().many())
    }

    pub fn parse(&self, input: String) -> Result<(T, String), ParseError> {
        (self.parser)(input)
    }
}

pub fn char(expected: char) -> Parser<char> {
    Parser::new(move |input| {
        let mut chars = input.chars();
        match chars.next() {
            Some(c) if c == expected => Ok((c, chars.collect())),
            _ => Err(ParseError::CharacterMismatch),
        }
    })
}

#[test]
fn parses_chars() {
    assert_eq!(
        char('a').parse("abc".to_string()),
        Ok(('a', "bc".to_string()))
    );

    assert_eq!(
        char('1').parse("123".to_string()),
        Ok(('1', "23".to_string()))
    );

    assert_eq!(
        char('f').parse("abc".to_string()),
        Err(ParseError::CharacterMismatch)
    );
}

pub fn list(mut allowed: impl Iterator<Item = char>) -> Parser<char> {
    if let Some(next) = allowed.next() {
        return char(next).or(list(allowed));
    } else {
        Parser::empty(ParseError::CharacterMismatch)
    }
}

#[test]
fn parses_lists() {
    assert_eq!(
        list('a'..='z').parse("abc".to_string()),
        Ok(('a', "bc".to_string()))
    );

    assert_eq!(
        list('a'..='z').parse("Abc".to_string()),
        Err(ParseError::CharacterMismatch)
    );
}

pub fn whitespace() -> Parser<char> {
    list([' ', '\n', '\r', '\t'].into_iter())
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

#[test]
fn alphanumerics_parse() {
    assert_eq!(
        alphanum().parse("Hello, world!".to_string()),
        Ok(('H', "ello, world!".to_string()))
    );

    assert_eq!(
        alphanum().parse(" Hello, world!".to_string()),
        Err(ParseError::CharacterMismatch)
    );
}

#[test]
fn parses_many() {
    assert_eq!(
        letter().many().parse("fuckshit123".to_string()),
        Ok((
            vec!['f', 'u', 'c', 'k', 's', 'h', 'i', 't'],
            "123".to_string()
        ))
    );

    assert_eq!(
        letter().many().parse("123fuckshit".to_string()),
        Ok((vec![], "123fuckshit".to_string()))
    );
}

pub fn string(mut input: Chars) -> Parser<String> {
    if let Some(next) = input.next() {
        return char(next)
            .and(string(input))
            .map(|(a, b)| once(a).chain(b.chars()).collect());
    } else {
        Parser::pure("".to_string())
    }
}

pub fn symbol(input: Chars) -> Parser<String> {
    string(input).strip()
}

#[test]
fn parses_string() {
    assert_eq!(
        string("123".chars()).parse("1234".to_string()),
        Ok(("123".to_string(), "4".to_string()))
    );
}

pub fn identifier() -> Parser<String> {
    letter()
        .and(alphanum().many())
        .strip()
        .map(|(a, b)| once(a).chain(b).collect())
}

#[test]
fn parses_identifiers() {
    assert_eq!(
        identifier().parse("_myShit123".to_string()),
        Ok(("_myShit123".to_string(), "".to_string()))
    );

    assert_eq!(
        identifier().parse("123fuckshit".to_string()),
        Err(ParseError::CharacterMismatch)
    );
}
