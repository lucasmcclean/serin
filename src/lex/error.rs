#[derive(Debug)]
pub enum LexError {
    UnexpectedByte(u8, usize),
    UnterminatedComment,
}
