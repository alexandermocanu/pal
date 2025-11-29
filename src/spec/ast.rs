/// Describes any possible type expression.
#[derive(Clone, Debug)]
pub enum Type {
    Atomic(String),
}

/// Describes any possible expression, including left-recursive ones. There is no distinction in
/// the AST.
#[derive(Clone, Debug)]
pub enum Expression {
    StringLiteral(String),
    NumericLiteral(u64),
}

/// Describes any possible statement.
#[derive(Clone, Debug)]
pub enum Statement {
    Return(Expression),
}

/// Describes any top-level item. That is, any item that is defined at the top level of a module,
/// such as a function declaration or an extern function definition.
#[derive(Clone, Debug)]
pub enum Item {
    ExternFunctionDefinition(String, Vec<(String, Type)>, Type),
    FunctionDeclaration(String, Vec<(String, Type)>, Type, Vec<Statement>),
}

/// Describes an individual code module.
#[derive(Clone, Debug)]
pub struct Module(pub String, pub Vec<Item>);
