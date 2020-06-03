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
        let stmt = line.split("//").nth(0).unwrap_or_default().trim();
        match stmt {
            "" => None,
            s => Some(Instruction::Push {}),
            _ => None,
        }
    }
}

pub fn parse(content: String) -> ParseResult {
    let parser = Parser::new(&content);
    parser.parse()
}
