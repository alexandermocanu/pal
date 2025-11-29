pub mod ast;

use crate::parser::*;
use ast::*;

pub fn num_literal() -> Parser<Expression> {
    digit()
        .many()
        .map(|s| Expression::NumericLiteral(s.iter().collect::<String>().parse::<u64>().unwrap()))
}

pub fn expression() -> Parser<Expression> {
    num_literal()
}

pub fn r#return() -> Parser<Statement> {
    symbol("return").right(expression()).map(Statement::Return)
}

pub fn statement() -> Parser<Statement> {
    r#return().left(symbol(";"))
}

pub fn function_declaration() -> Parser<Item> {
    symbol("fn")
        .right(identifier())
        .left(symbol("()"))
        .left(symbol("->"))
        .left(identifier())
        .chain(between(symbol("{"), statement().many(), symbol("}")))
        .map(|(a, b)| Item::FunctionDeclaration(a, b))
}

pub fn item() -> Parser<Item> {
    function_declaration()
}

pub fn module() -> Parser<Module> {
    item().many().map(Module)
}
