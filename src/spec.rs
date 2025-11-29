use crate::parser::*;

#[derive(Clone, Debug)]
pub enum Statement {
    FnCall(String, String),
    Return(u32),
}

#[derive(Clone, Debug)]
pub enum Item {
    FnDef(String, Vec<Statement>),
}

#[derive(Clone, Debug)]
pub struct Program(pub Vec<Item>);

pub fn fn_call() -> Parser<Statement> {
    identifier()
        .and(
            alphanum()
                .or(whitespace())
                .many()
                .between(symbol("\"".chars()), symbol("\"".chars()))
                .between(symbol("(".chars()), symbol(")".chars())),
        )
        .map(|(a, b)| Statement::FnCall(a, b.iter().collect()))
}

pub fn return_stat() -> Parser<Statement> {
    symbol("return".chars())
        .right(
            digit()
                .many()
                .map(|nums| nums.iter().collect::<String>().parse::<u32>().unwrap()),
        )
        .map(Statement::Return)
}

pub fn statement() -> Parser<Statement> {
    fn_call().or(return_stat()).left(symbol(";".chars()))
}

pub fn fn_def() -> Parser<Item> {
    symbol("fn".chars())
        .right(identifier())
        .left(symbol("()".chars()))
        .left(symbol("->".chars()).and(identifier()))
        .and(
            statement()
                .many()
                .between(symbol("{".chars()), symbol("}".chars())),
        )
        .map(|(a, b)| Item::FnDef(a, b))
}

pub fn item() -> Parser<Item> {
    fn_def()
}

pub fn program() -> Parser<Program> {
    item().many().map(Program)
}
