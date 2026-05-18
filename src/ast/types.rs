#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Named(String),
    Variable(String),
    Function { from: Box<Type>, to: Box<Type> },
}
