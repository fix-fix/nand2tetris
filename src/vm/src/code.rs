use crate::instruction::{PushPop::*, *};
use crate::parser::{Command, ParseResult};

pub fn generate_code(parse_result: ParseResult) -> String {
    let body: String = parse_result
        .commands
        .into_iter()
        .enumerate()
        .filter_map(|(cmd_index, cmd)| generate(&cmd, cmd_index))
        .collect();
    format!(
        "///@module-start '{module}'
{body}\
///@module-end '{module}'",
        module = parse_result.module,
        body = body
    )
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

fn generate(cmd: &Command, cmd_index: usize) -> Option<String> {
    // println!("generate: {:?}", cmd.inst);
    let asm: Option<String> = match &cmd.inst {
        Instruction::Function(name, n_args) => Some(generate_inst_function(name, *n_args)),
        Instruction::Call(name, n_args) => Some(generate_inst_call(
            name,
            *n_args,
            cmd.module_name,
            cmd_index,
        )),
        Instruction::Return() => Some(generate_inst_return(cmd)),
        Instruction::PushPop(x) => generate_inst_pushpop(x, cmd),
        Instruction::Arithmetic(cmd_type) => generate_inst_arithmetic(cmd_type, cmd_index),
        Instruction::Label(label) => Some(generate_inst_label(label, cmd)),
        Instruction::Goto(label) => Some(generate_inst_goto(label, cmd)),
        Instruction::IfGoto(label) => Some(generate_inst_ifgoto(label, cmd)),
        #[allow(unreachable_patterns)]
        _ => None,
    };
    if let Some(code) = asm {
        Some(format!("// {}\n{}\n", cmd.raw, code))
    } else {
        println!("Unknown instruction: {:?}", cmd);
        None
    }
}

pub fn generate_bootstrap(program_name: &str) -> String {
    format_asm!(
        "\
///@program-bootstrap-start
@256
D=A
@SP
M=D
{call_init}
///@program-bootstrap-end\
",
        call_init = generate_inst_call("Sys.init", 0, program_name, 0),
    )
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

fn get_pointer_base(segment: &str) -> Option<(&'static str, bool)> {
    match segment {
        "argument" => Some(("ARG", true)),
        "local" => Some(("LCL", true)),
        "temp" => Some(("5", false)),
        "this" => Some(("THIS", true)),
        "that" => Some(("THAT", true)),
        _ => None,
    }
}

fn generate_inst_pushpop(inst: &PushPopInstruction, cmd: &Command) -> Option<String> {
    match (&inst.inst_type, inst.segment.as_str()) {
        (Push { .. }, "static") => Some(format_asm!(
            "\
@{label}.{addr}
D=M
@SP
A=M
M=D
@SP
M=M+1\
",
            label = cmd.module_name,
            addr = inst.addr,
        )),
        (Pop { .. }, "static") => Some(format_asm!(
            "\
@SP
M=M-1
@SP
A=M
D=M
@{label}.{addr}
M=D\
",
            label = cmd.module_name,
            addr = inst.addr,
        )),
        (Push { .. }, "pointer") => {
            let label = if inst.addr == 0 { "THIS" } else { "THAT" };
            Some(format_asm!(
                "\
@{label}
D=M
@SP
A=M
M=D
@SP
M=M+1\
",
                label = label
            ))
        }
        (Pop { .. }, "pointer") => {
            let label = if inst.addr == 0 { "THIS" } else { "THAT" };
            Some(format_asm!(
                "\
@SP
M=M-1
@SP
A=M
D=M
@{label}
M=D\
",
                label = label
            ))
        }
        (Push { .. }, "constant") => Some(format_asm!(
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
        (Push { .. }, segment) => {
            let (pointer_base, is_relative) = get_pointer_base(segment)?;
            let asm_set_segment = if is_relative { "A=D+M" } else { "A=D+A" };
            Some(format_asm!(
                "\
@{addr} // @{pointer_base} + offset
D=A
@{pointer_base}
{asm_set_segment}
D=M
@SP
A=M
M=D
@SP
M=M+1\
",
                addr = inst.addr,
                pointer_base = pointer_base,
                asm_set_segment = asm_set_segment,
            ))
        }
        (Pop { .. }, segment) => {
            let (pointer_base, is_relative) = get_pointer_base(segment)?;
            let asm_set_segment = if is_relative { "D=D+M" } else { "D=D+A" };
            Some(format_asm!(
                "\
@SP
M=M-1
@SP
A=M
D=M
@R13
M=D
@{addr} // @{pointer_base} + offset
D=A
@{pointer_base}
{asm_set_segment}
@R14
M=D
@R13
D=M
@R14
A=M
M=D\
",
                addr = inst.addr,
                pointer_base = pointer_base,
                asm_set_segment = asm_set_segment,
            ))
        }
    }
}

fn generate_inst_label(label: &str, _cmd: &Command) -> String {
    format_asm!("({label})", label = label,)
}

fn generate_inst_goto(label: &str, _cmd: &Command) -> String {
    format_asm!(
        "\
@{label}
0;JMP\
",
        label = label,
    )
}

fn generate_inst_ifgoto(label: &str, _cmd: &Command) -> String {
    format_asm!(
        "\
@SP
M=M-1
@SP
A=M
D=M
@{label}
D;JNE\
",
        label = label,
    )
}

fn generate_inst_function(name: &str, n_args: usize) -> String {
    let push_parts: Vec<_> = [format_asm!(
        "@{value}\n{push_A}",
        push_A = _generate_push_stack("A"),
        value = 0
    )
    .as_str()]
    .repeat(n_args)
    .iter()
    .map(ToString::to_string)
    .collect();
    [&[format_asm!("({name})", name = name)], &push_parts[..]]
        .concat()
        .join("\n")
}

fn generate_inst_return(_cmd: &Command) -> String {
    let restore_label = |label: &str| {
        format!(
            "\
@R14
M=M-1
A=M
D=M
@{label}
M=D\
",
            label = label
        )
    };
    format!(
        "\
@3563 // debug return breakpoint, 0xdeb
@SP
A=M-1
D=M
@R15 // keep return value in register, to avoid overriding RIP when function has no local vars
M=D
@ARG
D=M
@R13
M=D
@ARG
D=M+1
@SP
M=D
@LCL // endFrame start
D=M
@R14
M=D // endFrame end
{}
{}
{}
{}
@R14
M=M-1
A=M
D=M
@R14
M=D
@R15
D=M
@R13
A=M
M=D
@R14
A=M
0;JMP\
",
        restore_label("THAT"),
        restore_label("THIS"),
        restore_label("ARG"),
        restore_label("LCL"),
    )
}

fn _generate_push_stack(reg: &str) -> String {
    format_asm!(
        "\
D={reg}
@SP
A=M
M=D
@SP
M=M+1",
        reg = reg
    )
}

fn generate_inst_call(name: &str, n_args: usize, program_name: &str, cmd_index: usize) -> String {
    let label_return = format!("{}$ret.{}", program_name, cmd_index);
    format_asm!(
        "\
@{label_return}
{push_A}
@LCL
{push_M}
@ARG
{push_M}
@THIS
{push_M}
@THAT
{push_M}
@5
D=A
@{n_args}
D=D+A
@SP
D=M-D
@ARG
M=D
@SP
D=M
@LCL
M=D
@{name} // goto function
0;JMP
({label_return})\
",
        label_return = label_return,
        push_A = _generate_push_stack("A"),
        push_M = _generate_push_stack("M"),
        n_args = n_args,
        name = name,
    )
}
