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
    Some(format!(
        "// {}\n{}\n",
        cmd.raw,
        match cmd {
            _ => "-".to_string(),
            _ => {
                println!("Unknown instruction: {:?}", cmd);
                return None;
            }
        }
    ))
}
