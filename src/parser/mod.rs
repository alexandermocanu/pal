pub mod error;
pub mod generators;

use std::{iter::once, sync::Arc};

use error::ParseError;

#[derive(Clone)]
pub struct Parser<T> {
    parser: Arc<dyn Fn(String) -> Result<(T, String), ParseError>>,
}

impl<T: 'static> Parser<T> {
    pub fn new(parser: impl Fn(String) -> Result<(T, String), ParseError> + 'static) -> Parser<T> {
        Parser {
            parser: Arc::new(parser),
        }
    }

    pub fn lazy(producer: impl Fn() -> Parser<T> + 'static) -> Parser<T> {
        Parser::new(move |input| producer().parse(input))
    }

    // Functor
    pub fn map<O: 'static>(self, f: impl Fn(T) -> O + 'static) -> Parser<O> {
        Parser::new(move |input| self.parse(input).map(|(result, input)| (f(result), input)))
    }

    // Applicative
    pub fn pure(value: T) -> Parser<T>
    where
        T: Clone,
    {
        Parser::new(move |input| Ok((value.clone(), input)))
    }

    pub fn chain<O: 'static>(self, other: Parser<O>) -> Parser<(T, O)> {
        Parser::new(move |input| {
            self.parse(input).and_then(|(result_a, input)| {
                other
                    .parse(input)
                    .map(|(result_b, input)| ((result_a, result_b), input))
            })
        })
    }

    pub fn left<O: 'static>(self, other: Parser<O>) -> Parser<T> {
        self.chain(other).map(|(result, _)| result)
    }

    pub fn right<O: 'static>(self, other: Parser<O>) -> Parser<O> {
        self.chain(other).map(|(_, result)| result)
    }

    // Alternative
    pub fn empty(value: ParseError) -> Parser<T> {
        Parser::new(move |_| Err(value.clone()))
    }

    pub fn or(self, other: Parser<T>) -> Parser<T> {
        Parser::new(move |input| self.parse(input.clone()).or_else(|_| other.parse(input)))
    }

    pub fn maybe(self) -> Parser<Option<T>>
    where
        T: Clone,
    {
        self.map(Some).or(Parser::pure(None))
    }

    pub fn many(self) -> Parser<Vec<T>>
    where
        T: Clone,
    {
        self.clone()
            .chain(Parser::lazy(move || self.clone().many()))
            .map(|(x, xs)| once(x).chain(xs).collect())
            .or(Parser::pure(vec![]))
    }

    pub fn some(self) -> Parser<Vec<T>>
    where
        T: Clone,
    {
        self.clone()
            .chain(self.many())
            .map(|(x, xs)| Some(x).into_iter().chain(xs.into_iter()).collect())
    }

    pub fn parse(&self, input: impl ToString) -> Result<(T, String), ParseError> {
        (self.parser)(input.to_string())
    }
}

#[test]
fn functor_is_mappable() {
    assert_eq!(
        Parser::pure(()).map(|_| 32u32).parse("123"),
        Ok((32u32, "123".to_string()))
    );
}

#[test]
fn applicative_is_pure() {
    assert_eq!(Parser::pure(()).parse("123"), Ok(((), "123".to_string())));
}

#[test]
fn applicatives_can_chain() {
    let p1: Parser<()> = Parser::pure(());
    let p2: Parser<()> = Parser::empty(ParseError::CharacterMismatch {
        expected: vec![],
        found: None,
    });

    assert_eq!(
        p1.clone().chain(p1.clone()).parse(""),
        Ok((((), ()), "".to_string()))
    );

    assert_eq!(
        p1.clone().chain(p2.clone()).parse(""),
        Err(ParseError::CharacterMismatch {
            expected: vec![],
            found: None
        })
    );

    assert_eq!(
        p2.clone().chain(p1.clone()).parse(""),
        Err(ParseError::CharacterMismatch {
            expected: vec![],
            found: None
        })
    );
}

#[test]
fn applicatives_can_chain_left_and_right() {
    let p1: Parser<()> = Parser::pure(());
    let p2: Parser<()> = Parser::pure(());
    assert_eq!(
        p1.clone().left(p2.clone()).parse(""),
        Ok(((), "".to_string()))
    );
    assert_eq!(
        p1.clone().right(p2.clone()).parse(""),
        Ok(((), "".to_string()))
    );
}

#[test]
fn alternative_is_empty() {
    assert_eq!(
        Parser::<()>::empty(ParseError::CharacterMismatch {
            expected: vec![],
            found: None
        })
        .parse("".to_string()),
        Err(ParseError::CharacterMismatch {
            expected: vec![],
            found: None
        })
    );
}

#[test]
fn alternative_maybe_exists() {
    assert_eq!(
        Parser::pure(()).maybe().parse(""),
        Ok((Some(()), "".to_string()))
    );

    assert_eq!(
        Parser::<()>::empty(ParseError::CharacterMismatch {
            expected: vec![],
            found: None
        })
        .maybe()
        .parse(""),
        Ok((None, "".to_string()))
    );
}

#[test]
fn alternative_many() {
    let char_a = Parser::new(move |input| {
        let mut chars = input.chars();
        match chars.next() {
            Some('a') => Ok(('a', chars.collect())),
            found => Err(ParseError::CharacterMismatch {
                expected: vec!['a'],
                found,
            }),
        }
    });

    assert_eq!(
        char_a.clone().many().parse("aaab"),
        Ok((vec!['a', 'a', 'a'], "b".to_string()))
    );

    assert_eq!(
        char_a.clone().many().parse("bbbb"),
        Ok((vec![], "bbbb".to_string()))
    );
}

#[test]
fn alternative_some() {
    let char_a = Parser::new(move |input| {
        let mut chars = input.chars();
        match chars.next() {
            Some('a') => Ok(('a', chars.collect())),
            found => Err(ParseError::CharacterMismatch {
                expected: vec!['a'],
                found,
            }),
        }
    });

    assert_eq!(
        char_a.clone().some().parse("aaab"),
        Ok((vec!['a', 'a', 'a'], "b".to_string()))
    );

    assert_eq!(
        char_a.clone().some().parse("bbbb"),
        Err(ParseError::CharacterMismatch {
            expected: vec!['a'],
            found: Some('b')
        })
    );
}
