use crate::ast::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Integer(i64),
    Boolean(bool),
    Identifier(String),
    Lambda {
        parameter: String,
        annotation: Option<Type>,
        body: Box<Expression>,
    },
    Application {
        function: Box<Expression>,
        argument: Box<Expression>,
    },
    Let {
        name: String,
        value: Box<Expression>,
        body: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Tuple(Vec<Expression>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}
