use crate::ast::Type;

#[derive(Clone, Debug)]
pub enum Expression {
    Integer(i64),
    Boolean(bool),
    Identifier(String),
    Lambda {
        parameter: String,
        parameter_type: Option<Type>,
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

#[derive(Clone, Copy, Debug)]
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
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOperator {
    Not,
    Negate,
}
