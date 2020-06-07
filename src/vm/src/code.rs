use crate::instruction::*;
use crate::parser::{Command, ParseResult};

pub fn generate_code(parse_result: ParseResult) -> String {
    parse_result
        .commands
        .into_iter()
        .enumerate()
        .filter_map(|(cmd_index, cmd)| generate(cmd, cmd_index))
        .collect()
}

// Wrap format! to provide some builtins
macro_rules! format_asm {
    ($fmt:expr, $($arg:tt)*) => {{
        format!(
            // "Consume" builtins agruments, to allow them to be unused in a caller
            // See: https://stackoverflow.com/a/41911995/656914
            concat!($fmt, "{TRUE:.0}{FALSE:.0}"),
            TRUE="-1",
            FALSE="0",
            $($arg)*
        )
    }}
}

fn generate(cmd: Command, cmd_index: usize) -> Option<String> {
    // println!("generate: {:?}", cmd.inst);
    let asm: Option<String> = match &cmd.inst {
        Instruction::PushPop(x) => generate_inst_pushpop(x, cmd_index),
        Instruction::Arithmetic(cmd_type) => generate_inst_arithmetic(cmd_type, cmd_index),
    };
    if let Some(code) = asm {
        Some(format!("// {}\n{}\n", cmd.raw, code))
    } else {
        println!("Unknown instruction: {:?}", cmd);
        None
    }
}

fn generate_inst_arithmetic(inst: &str, cmd_index: usize) -> Option<String> {
    match inst {
        "add" => Some(format_asm!(
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
        "sub" => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
M=M-D\
",
        )),
        "neg" => Some(format_asm!(
            "\
@SP
A=M-1
M=-M\
",
        )),
        "eq" => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
D=M-D
@{label_prefix}
D;JEQ
@SP
A=M-1
M={FALSE}
@{label_prefix}_CONT
0;JMP
({label_prefix})
@SP
A=M-1
M={TRUE}
({label_prefix}_CONT)
",
            label_prefix = format!("EQ_LABEL_{}", cmd_index)
        )),
        "gt" => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
D=M-D
@{label_prefix}
D;JGT
@SP
A=M-1
M={FALSE}
@{label_prefix}_CONT
0;JMP
({label_prefix})
@SP
A=M-1
M={TRUE}
({label_prefix}_CONT)
",
            label_prefix = format!("JGT_LABEL_{}", cmd_index)
        )),
        "lt" => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
D=M-D
@{label_prefix}
D;JLT
@SP
A=M-1
M={FALSE}
@{label_prefix}_CONT
0;JMP
({label_prefix})
@SP
A=M-1
M={TRUE}
({label_prefix}_CONT)
",
            label_prefix = format!("JLT_LABEL_{}", cmd_index)
        )),
        "not" => Some(format_asm!(
            "\
@SP
A=M-1
M=!M\
",
        )),
        "and" => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
M=D&M\
",
        )),
        "or" => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
A=A-1
M=D|M\
",
        )),
        _ => None,
    }
}

fn generate_inst_pushpop(inst: &PushPopInstruction, _cmd_index: usize) -> Option<String> {
    match (&inst.inst_type, inst.segment.as_str()) {
        (PushPop::Push, "constant") => Some(format_asm!(
            "\
@{addr}
D=A
@SP
A=M
M=D
@SP
M=M+1\
",
            addr = inst.addr
        )),
        _ => None,
    }
}
