#[derive(Debug)]
pub enum Instruction {
    Push(String, u16),
    Pop(String, u16),
    Arithmetic(String),
}
