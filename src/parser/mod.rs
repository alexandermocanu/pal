pub mod error;
pub mod generators;

pub use generators::*;
use std::{iter::once, sync::Arc};

use error::ParseError;

/// A generic parser for pal.
#[derive(Clone)]
pub struct Parser<T> {
    parser: Arc<dyn Fn(String) -> Result<(T, String), ParseError>>,
}

impl<T: 'static> Parser<T> {
    /// Creates a new parser from a given function, which parses a given [`String`] and returns
    /// either a result and the rest of the input, or a parsing error.
    pub fn new(parser: impl Fn(String) -> Result<(T, String), ParseError> + 'static) -> Parser<T> {
        Parser {
            parser: Arc::new(parser),
        }
    }

    /// Makes the parser that is moved into the closure lazily evaulated, meaning it only gets
    /// initialized when you attempt to parse.
    pub fn lazy(producer: impl Fn() -> Parser<T> + 'static) -> Parser<T> {
        Parser::new(move |input| producer().parse(input))
    }

    // Functor
    /// Maps a [`Parser<T>`] to a [`Parser<O>`] with a function f such that `fn(T) -> O`.
    pub fn map<O: 'static>(self, f: impl Fn(T) -> O + 'static) -> Parser<O> {
        Parser::new(move |input| self.parse(input).map(|(result, input)| (f(result), input)))
    }

    // Applicative
    /// Returns a [`Parser<T>`] that always returns `Ok((T, String))`.
    pub fn pure(value: T) -> Parser<T>
    where
        T: Clone,
    {
        Parser::new(move |input| Ok((value.clone(), input)))
    }

    /// Chains two parsers together such that the return Parser expects [`Parser<O>`] to follow
    /// [`Parser<T>`].
    pub fn chain<O: 'static>(self, other: Parser<O>) -> Parser<(T, O)> {
        Parser::new(move |input| {
            self.parse(input).and_then(|(result_a, input)| {
                other
                    .parse(input)
                    .map(|(result_b, input)| ((result_a, result_b), input))
            })
        })
    }

    /// Chains two [`Parser`]s together and drops the left result.
    pub fn left<O: 'static>(self, other: Parser<O>) -> Parser<T> {
        self.chain(other).map(|(result, _)| result)
    }

    /// Chains two [`Parser`]s together and drops the right result.
    pub fn right<O: 'static>(self, other: Parser<O>) -> Parser<O> {
        self.chain(other).map(|(_, result)| result)
    }

    // Alternative
    /// Returns a [`Parser<T>`] that always returns `Err(ParseError)`.
    pub fn empty(value: ParseError) -> Parser<T> {
        Parser::new(move |_| Err(value.clone()))
    }

    /// Creates a [`Parser`] that attempts the given [`Parser`] when the calling [`Parser`] fails.
    /// Errors are ordered and higher ordering variants are prioritized.
    /// The choice is as follows:
    /// ```rs
    /// parse_error_a.max(parse_error_b)
    /// ```
    pub fn or(self, other: Parser<T>) -> Parser<T> {
        Parser::new(move |input| {
            self.parse(input.clone()).or_else(|parse_error_a| {
                other
                    .parse(input)
                    .map_err(|parse_error_b| parse_error_a.max(parse_error_b))
            })
        })
    }

    /// Creates a [`Parser`] that wraps a value in [`Option<T>`]. Returns `Some(T)` when the parser
    /// succeeds, otherwise returns `None`.
    pub fn maybe(self) -> Parser<Option<T>>
    where
        T: Clone,
    {
        self.map(Some).or(Parser::pure(None))
    }

    /// Creates a [`Parser`] that matches on zero or many possibilities.
    pub fn many(self) -> Parser<Vec<T>>
    where
        T: Clone,
    {
        self.clone()
            .chain(Parser::lazy(move || self.clone().many()))
            .map(|(x, xs)| once(x).chain(xs).collect())
            .or(Parser::pure(vec![]))
    }

    /// Creates a [`Parser`] that matches on one or many possibilities.
    /// This is equivalent to the following (omitting clones):
    /// ```rs
    /// parser.chain(parser.many()).map(merge)
    /// ```
    pub fn some(self) -> Parser<Vec<T>>
    where
        T: Clone,
    {
        self.clone()
            .chain(self.many())
            .map(|(x, xs)| Some(x).into_iter().chain(xs.into_iter()).collect())
    }

    /// Consumes a [`Parser`] with any type that implements [`ToString`] and returns the result.
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
    let p2: Parser<()> = Parser::empty(ParseError::Unit);

    assert_eq!(
        p1.clone().chain(p1.clone()).parse(""),
        Ok((((), ()), "".to_string()))
    );

    assert_eq!(
        p1.clone().chain(p2.clone()).parse(""),
        Err(ParseError::Unit)
    );

    assert_eq!(
        p2.clone().chain(p1.clone()).parse(""),
        Err(ParseError::Unit)
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
        Parser::<()>::empty(ParseError::Unit).parse("".to_string()),
        Err(ParseError::Unit)
    );
}

#[test]
fn alternative_maybe_exists() {
    assert_eq!(
        Parser::pure(()).maybe().parse(""),
        Ok((Some(()), "".to_string()))
    );

    assert_eq!(
        Parser::<()>::empty(ParseError::Unit).maybe().parse(""),
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
                expected: Some('a'),
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
                expected: Some('a'),
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
            expected: Some('a'),
            found: Some('b')
        })
    );
}
