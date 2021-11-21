use std::default::Default;

use crate::instruction::*;

#[derive(Debug, Clone)]
pub struct Command<'a> {
    pub inst: Instruction,
    pub raw: String,
    pub module_name: &'a str,
}

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub commands: Vec<Command<'a>>,
    pub module: String,
}

#[derive(Debug, Default, Clone)]
pub struct ParserContext {
    pub function_name: Option<String>,
}

#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a str,
    filename: &'a str,
    context: ParserContext,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str, filename: &'a str) -> Self {
        Parser {
            input,
            filename,
            context: Default::default(),
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        let mut commands = Vec::<Command>::new();
        for line in self.input.lines() {
            if let Some(inst) = self.parse_line(line) {
                commands.push(Command {
                    inst,
                    raw: line.into(),
                    module_name: self.filename,
                })
            }
        }
        ParseResult {
            commands,
            module: self.filename.into(),
        }
    }

    fn parse_line(&mut self, line: &str) -> Option<Instruction> {
        let cleaned = line.split("//").next().unwrap_or_default().trim();
        let cmds: Vec<&str> = cleaned.split_whitespace().collect();
        let result = match cmds[..] {
            ["call", label, n_args] => Some(Instruction::Call(
                label.into(),
                str::parse::<usize>(n_args).ok()?,
            )),
            ["function", label, n_args] => {
                self.context.function_name = Some(label.into());
                Some(Instruction::Function(
                    label.into(),
                    str::parse::<usize>(n_args).ok()?,
                ))
            }
            ["return"] => Some(Instruction::Return()),
            ["label", label] => Some(Instruction::Label(
                label.into(),
                self.context.function_name.clone(),
            )),
            ["goto", label] => Some(Instruction::Goto(
                label.into(),
                self.context.function_name.clone(),
            )),
            ["if-goto", label] => Some(Instruction::IfGoto(
                label.into(),
                self.context.function_name.clone(),
            )),
            [arith] => Some(Instruction::Arithmetic(arith.into())),
            [cmd1, cmd2, cmd3] => match cmd1 {
                "push" | "pop" => {
                    let inst_type = match cmd1 {
                        "push" => PushPop::Push,
                        "pop" => PushPop::Pop,
                        _ => unreachable!(),
                    };
                    Some(Instruction::PushPop(PushPopInstruction {
                        segment: cmd2.into(),
                        addr: str::parse::<u16>(cmd3).ok()?,
                        inst_type,
                    }))
                }
                _ => None,
            },
            _ => None,
        };
        if result.is_none() && !cleaned.is_empty() {
            eprintln!("Unable to parse line: {}", line);
        }
        result
    }
}

pub fn create<'a>(content: &'a str, filename: &'a str) -> Parser<'a> {
    Parser::new(content, filename)
}
