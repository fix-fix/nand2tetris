use crate::instruction::*;

#[derive(Debug)]
pub struct Command {
    pub inst: Instruction,
    pub raw: String,
}

#[derive(Debug)]
pub struct ParseResult {
    pub commands: Vec<Command>,
    pub inst_counter: i32,
}

pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input }
    }

    pub fn parse(&mut self) -> ParseResult {
        let mut inst_counter = 0;
        let mut commands = Vec::<Command>::new();
        for line in self.input.lines() {
            if let Some(command) = Self::parse_line(line) {
                inst_counter += 1;
                commands.push(command);
            }
        }
        ParseResult {
            commands,
            inst_counter,
        }
    }

    fn parse_line(line: &str) -> Option<Command> {
        let stmt = line.split("//").nth(0).unwrap_or_default();
        match stmt {
            "" => None,
            x if x.starts_with("@") => {
                let address = str::parse::<i32>(&x[1..]).ok()?;
                Some(Command {
                    raw: x.into(),
                    inst: Instruction::AInstruction { address },
                })
            }
            x => {
                let dest = x.split("=").nth(0);
                let jump = x.split(";").nth(1);
                let comp = x
                    .replace(&format!("{}{}", dest.unwrap_or_default(), "="), "")
                    .replace(&format!("{}{}", ";", jump.unwrap_or_default()), "");
                Some(Command {
                    raw: x.into(),
                    inst: Instruction::CInstruction {
                        comp: Self::parse_comp(comp),
                        dest: Self::parse_dest(dest),
                        jump: Self::parse_jump(jump),
                    },
                })
            }
        }
    }

    fn parse_comp(s: String) -> Comp {
        make_comp(s)
    }

    fn parse_dest(s: Option<&str>) -> Dest {
        match s {
            Some("M") => Dest::M,
            Some("D") => Dest::D,
            Some("MD") => Dest::MD,
            Some("A") => Dest::A,
            Some("AM") => Dest::AM,
            Some("AD") => Dest::AD,
            Some("AMD") => Dest::AMD,
            _ => Dest::Null,
        }
    }

    fn parse_jump(s: Option<&str>) -> Jump {
        match s {
            Some("JGT") => Jump::JGT,
            Some("JEQ") => Jump::JEQ,
            Some("JGE") => Jump::JGE,
            Some("JLT") => Jump::JLT,
            Some("JNE") => Jump::JNE,
            Some("JLE") => Jump::JLE,
            Some("JMP") => Jump::JMP,
            _ => Jump::Null,
        }
    }
}

pub fn parse(content: String) -> ParseResult {
    let mut parser = Parser::new(&content);
    parser.parse()
}
