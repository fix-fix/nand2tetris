use vm::instruction::{PushPop, PushPopInstruction};

use crate::instruction::Instruction;

#[derive(Clone, Debug)]
pub struct CompilerInstruction {
    instruction: Instruction,
}

#[derive(Clone, Debug)]
pub struct WriteInst {
    instruction: CompilerInstruction,
}

impl WriteInst {
    pub fn new(instruction: CompilerInstruction) -> Self {
        Self { instruction }
    }

    pub fn code(&self) -> String {
        self.instruction.code()
    }

    pub fn instruction(&self) -> &Instruction {
        &self.instruction.instruction
    }
}

impl CompilerInstruction {
    pub fn new(instruction: Instruction) -> Self {
        Self { instruction }
    }

    #[allow(unreachable_patterns)]
    pub fn code(&self) -> String {
        match &self.instruction {
            Instruction::Arithmetic(op) => op.into(),
            Instruction::PushPop(PushPopInstruction {
                segment,
                addr: index,
                inst_type,
            }) => match inst_type {
                PushPop::Push => format!("push {} {}", segment, index),
                PushPop::Pop => format!("pop {} {}", segment, index),
            },
            Instruction::Label(label, _) => format!("label {}", label),
            Instruction::Goto(label, _) => format!("goto {}", label),
            Instruction::IfGoto(label, _) => format!("if-goto {}", label),
            Instruction::Function(name, n_locals) => format!("function {} {}", name, n_locals),
            Instruction::Return() => "return".to_string(),
            Instruction::Call(name, n_args) => format!("call {} {}", name, n_args),
            _ => unreachable!("Unreachable instruction: {:?}", self.instruction),
        }
    }
}

pub fn write_arith(op: &str) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::Arithmetic(op.into())))
}

pub fn write_not() -> WriteInst {
    write_arith("not")
}

pub fn write_pop(seg: &str, index: u16) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::PushPop(
        PushPopInstruction::new(PushPop::Pop, seg.into(), index),
    )))
}

pub fn write_push(seg: &str, index: u16) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::PushPop(
        PushPopInstruction::new(PushPop::Push, seg.into(), index),
    )))
}

pub fn write_function(name: String, n_locals: u16) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::Function(
        name,
        n_locals.into(),
    )))
}

pub fn write_call<S: std::fmt::Display>(name: S, n_args: usize) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::Call(
        name.to_string(),
        n_args,
    )))
}

pub fn write_return() -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::Return()))
}

pub fn write_label<S: std::fmt::Display>(label: S) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::Label(
        label.to_string(),
        None,
    )))
}

pub fn write_if<S: std::fmt::Display>(label: S) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::IfGoto(
        label.to_string(),
        None,
    )))
}

pub fn write_goto<S: std::fmt::Display>(label: S) -> WriteInst {
    WriteInst::new(CompilerInstruction::new(Instruction::Goto(
        label.to_string(),
        None,
    )))
}
