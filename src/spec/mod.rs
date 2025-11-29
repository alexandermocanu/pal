pub mod ast;

use crate::parser::*;
use ast::*;

pub fn atomic() -> Parser<Type> {
    identifier().map(Type::Atomic)
}

pub fn pointer() -> Parser<Type> {
    symbol("*").right(typ()).map(Box::new).map(Type::Pointer)
}

pub fn typ() -> Parser<Type> {
    Parser::lazy(|| atomic().or(pointer()))
}

pub fn str_literal() -> Parser<Expression> {
    between(
        symbol("\""),
        alphanum().many().qualify().map(Expression::StringLiteral),
        symbol("\""),
    )
}

pub fn num_literal() -> Parser<Expression> {
    // TODO: make this parser use an "and_then"/flatmap
    Parser::new(move |input| {
        match digit()
            .many()
            .qualify()
            .map(|str| str.parse::<u64>())
            .parse(input)
        {
            Ok((Ok(num), remaining)) => Ok((num, remaining)),
            _ => Err(error::ParseError::Unit),
        }
    })
    .map(Expression::NumericLiteral)
}

pub fn expression() -> Parser<Expression> {
    str_literal().or(num_literal())
}

pub fn function_call() -> Parser<Statement> {
    identifier()
        .chain(between(
            symbol("("),
            expression()
                .maybe()
                .chain(symbol(",").right(expression()).many()),
            symbol(")"),
        ))
        .map(|(name, (head, rest))| {
            Statement::FunctionCall(name, head.into_iter().chain(rest.into_iter()).collect())
        })
}

pub fn ret() -> Parser<Statement> {
    symbol("return").right(expression()).map(Statement::Return)
}

pub fn statement() -> Parser<Statement> {
    function_call().or(ret()).left(symbol(";"))
}

pub fn argument_parser() -> Parser<Vec<(String, Type)>> {
    let argument = identifier().left(symbol(":")).chain(typ());

    argument
        .clone()
        .maybe()
        .chain(symbol(",").right(argument).many())
        .left(symbol(",").maybe())
        .map(|(head, rest)| head.into_iter().chain(rest.into_iter()).collect())
}

pub fn extern_function_definition() -> Parser<Item> {
    symbol("ext")
        .chain(symbol("fn"))
        .right(identifier())
        .chain(between(symbol("("), argument_parser(), symbol(")")))
        .left(symbol("->"))
        .chain(typ())
        .map(|((a, b), c)| Item::ExternFunctionDefinition(a, b, c))
}

pub fn function_declaration() -> Parser<Item> {
    symbol("fn")
        .right(identifier())
        .chain(between(symbol("("), argument_parser(), symbol(")")))
        .left(symbol("->"))
        .chain(typ())
        .chain(between(symbol("{"), statement().many(), symbol("}")))
        .map(|(((a, b), c), d)| Item::FunctionDeclaration(a, b, c, d))
}

pub fn item() -> Parser<Item> {
    extern_function_definition().or(function_declaration())
}

pub fn module(name: String) -> Parser<Module> {
    item()
        .left(symbol(";"))
        .many()
        .map(move |items| Module(name.clone(), items))
}
