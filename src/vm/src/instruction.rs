#[derive(Debug)]
pub enum PushPop {
    Push,
    Pop,
}

#[derive(Debug)]
pub struct PushPopInstruction {
    pub segment: String,
    pub addr: u16,
    pub inst_type: PushPop,
}

#[derive(Debug)]
pub enum Instruction {
    PushPop(PushPopInstruction),
    Arithmetic(String),
    Label(String),
    Goto(String),
    IfGoto(String),
    Function(String, usize),
    Return(),
}
