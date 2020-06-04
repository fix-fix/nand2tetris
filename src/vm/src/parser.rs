use crate::instruction::*;

#[derive(Debug)]
pub struct Command {
    pub inst: Instruction,
    pub raw: String,
}

#[derive(Debug)]
pub struct ParseResult {
    pub commands: Vec<Command>,
}

pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input }
    }

    pub fn parse(&self) -> ParseResult {
        let mut commands = Vec::<Command>::new();
        for line in self.input.lines() {
            if let Some(inst) = Self::parse_line(line) {
                commands.push(Command {
                    inst,
                    raw: line.into(),
                })
            }
        }
        ParseResult { commands: commands }
    }

    fn parse_line(line: &str) -> Option<Instruction> {
        let cleaned = line.split("//").nth(0).unwrap_or_default().trim();
        let cmds: Vec<&str> = cleaned.split_whitespace().collect();
        match cmds[..] {
            [arith] => Some(Instruction::Arithmetic(arith.into())),
            [cmd1, cmd2, cmd3] => match cmd1 {
                "push" | "pop" => {
                    let cmd_type = match cmd1 {
                        "push" => Instruction::Push,
                        "pop" => Instruction::Pop,
                        _ => unreachable!(),
                    };
                    Some(cmd_type(cmd2.into(), str::parse::<u16>(&cmd3).ok()?))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn parse(content: String) -> ParseResult {
    let parser = Parser::new(&content);
    parser.parse()
}
