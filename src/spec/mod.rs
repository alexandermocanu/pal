pub mod ast;

use crate::parser::*;
use ast::*;

pub fn atomic() -> Parser<Type> {
    identifier().map(Type::Atomic)
}

pub fn typ() -> Parser<Type> {
    atomic()
}

pub fn str_literal() -> Parser<Expression> {
    between(
        symbol("\""),
        alphanum().many().qualify().map(Expression::StringLiteral),
        symbol("\""),
    )
}

pub fn num_literal() -> Parser<Expression> {
    digit()
        .many()
        .qualify()
        .map(|str| Expression::NumericLiteral(str.parse::<u64>().unwrap()))
}

pub fn expression() -> Parser<Expression> {
    str_literal().or(num_literal())
}

pub fn ret() -> Parser<Statement> {
    symbol("return").right(expression()).map(Statement::Return)
}

pub fn statement() -> Parser<Statement> {
    ret().left(symbol(";"))
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
