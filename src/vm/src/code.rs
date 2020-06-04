use crate::instruction::*;
use crate::parser::{Command, ParseResult};

pub fn generate_code(parse_result: ParseResult) -> String {
    parse_result
        .commands
        .into_iter()
        .filter_map(|com| generate(com))
        .collect()
}

fn generate(cmd: Command) -> Option<String> {
    // println!("generate: {:?}", cmd.inst);
    let asm: Option<String> = match &cmd.inst {
        Instruction::Push(seg, addr) => match seg.as_str() {
            "constant" => Some(format!(
                "\
@{addr}
D=A
@SP
A=M
M=D
@SP
M=M+1\
",
                addr = addr
            )),
            _ => None,
        },
        Instruction::Arithmetic(cmd_type) => match cmd_type.as_str() {
            "add" => Some(format!(
                "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
M=D+M\
",
            )),
            _ => None,
        },

        _ => None,
    };
    if let Some(code) = asm {
        Some(format!("// {}\n{}\n", cmd.raw, code))
    } else {
        println!("Unknown instruction: {:?}", cmd);
        None
    }
}
