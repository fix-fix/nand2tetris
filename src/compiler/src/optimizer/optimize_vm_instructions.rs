use crate::codegen::WriteInst;

use crate::instruction::Instruction;

pub fn opimize_vm_instructions(instructions: &mut Vec<WriteInst>) {
    let mut out = Vec::<WriteInst>::with_capacity(instructions.len());
    let mut insts_iter = instructions.iter().peekable();

    while let Some(&wi) = insts_iter.peek() {
        let inst = wi.instruction();
        match inst {
            Instruction::Arithmetic(op) => {
                insts_iter.next();
                match (op.as_str(), insts_iter.peek().map(|i| i.instruction())) {
                    ("not", Some(Instruction::Arithmetic(op2))) if op2 == "not" => {
                        insts_iter.next();
                        continue;
                    }
                    _ => {}
                }
            }
            _ => {
                insts_iter.next();
            }
        }
        out.push(wi.to_owned());
    }

    *instructions = out;
}
