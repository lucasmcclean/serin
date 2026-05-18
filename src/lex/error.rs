#[derive(Debug)]
pub enum Error {
    UnexpectedByte(u8, usize),
    UnterminatedComment,
    IntegerOverflow(String),
}
