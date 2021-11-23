use crate::instruction::Instruction;

use crate::codegen::WriteInst;

pub fn opimize_vm_instructions(instructions: &mut Vec<WriteInst>) {
    let mut out = Vec::<WriteInst>::with_capacity(instructions.len());
    let mut insts_iter = instructions.iter().peekable();

    while let Some(wi) = insts_iter.next() {
        let inst = wi.instruction();
        match inst {
            Instruction::Arithmetic(op) => {
                match (op.as_str(), insts_iter.peek().map(|i| i.instruction())) {
                    ("not", Some(Instruction::Arithmetic(op2))) if op2 == "not" => {
                        insts_iter.next();
                        continue;
                    }
                    _ => {}
                }
            }
            Instruction::Goto(goto_label, ..) => {
                if let Some(Instruction::Label(label, ..)) =
                    insts_iter.peek().map(|i| i.instruction())
                {
                    if label == goto_label {
                        continue;
                    }
                }
            }
            _ => {}
        }
        out.push(wi.to_owned());
    }

    *instructions = out;
}
