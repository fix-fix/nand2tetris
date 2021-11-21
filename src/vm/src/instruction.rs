#[derive(Debug, Clone)]
pub enum PushPop {
    Push,
    Pop,
}

#[derive(Debug, Clone)]
pub struct PushPopInstruction {
    pub segment: String,
    pub addr: u16,
    pub inst_type: PushPop,
}

impl PushPopInstruction {
    pub fn new(inst_type: PushPop, segment: String, addr: u16) -> Self {
        Self {
            segment,
            addr,
            inst_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    PushPop(PushPopInstruction),
    Arithmetic(String),
    Label(String, Option<String>),
    Goto(String, Option<String>),
    IfGoto(String, Option<String>),
    Function(String, usize),
    Return(),
    Call(String, usize),
}
