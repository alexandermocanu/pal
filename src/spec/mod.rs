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

pub fn function_declaration() -> Parser<Item> {
    symbol("fn")
        .right(identifier())
        .left(symbol("()"))
        .left(symbol("->"))
        .chain(typ())
        .chain(between(symbol("{"), statement().many(), symbol("}")))
        .map(|((a, b), c)| Item::FunctionDeclaration(a, b, c))
}

pub fn item() -> Parser<Item> {
    function_declaration()
}

pub fn module(name: String) -> Parser<Module> {
    item().many().map(move |items| Module(name.clone(), items))
}
