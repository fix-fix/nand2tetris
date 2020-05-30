use crate::instruction::*;

#[derive(Debug)]
pub struct Command {
    pub inst: Instruction,
    pub raw: String,
}

#[derive(Debug)]
pub struct ParseResult {
    pub commands: Vec<Command>,
    pub inst_counter: u16,
}

pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input }
    }

    pub fn parse(&self) -> ParseResult {
        let (_, symbols_parsed) = self.parse_input(None);
        println!("symbols: {:?}", symbols_parsed);
        let symbols = self.build_symbols(symbols_parsed);
        let (result, _) = self.parse_input(Some(&symbols));
        result
    }

    pub fn parse_input(&self, symbols: Option<&SymbolTable>) -> (ParseResult, SymbolTable) {
        let mut inst_counter = 0u16;
        let mut commands = Vec::<Command>::new();
        let mut symbols_parsed = SymbolTable::new();
        for line in self.input.lines() {
            if let Some(command) = Self::parse_line(line, symbols) {
                match &command.inst {
                    Instruction::LInstruction { label } => {
                        // dbg!("LInstruction", label, inst_counter);
                        symbols_parsed.insert(label.into(), inst_counter);
                    }
                    _ => {
                        inst_counter += 1;
                        commands.push(command);
                    }
                };
            } else {
                println!("Unable to parse line: {}", line);
            }
        }
        (
            ParseResult {
                commands,
                inst_counter,
            },
            symbols_parsed,
        )
    }

    fn parse_line(line: &str, symbols_maybe: Option<&SymbolTable>) -> Option<Command> {
        let stmt = line.split("//").nth(0).unwrap_or_default().trim();
        // println!("parse_line: {:?}, {:?}", &line, &stmt);
        match stmt {
            "" => None,
            x if x.starts_with("@") => {
                let label: String = x.chars().skip(1).collect();
                let address = str::parse::<u16>(&label).ok();
                Some(Command {
                    raw: x.into(),
                    inst: match address {
                        Some(address) => Instruction::AInstruction {
                            address: AInstAddress::Address(address),
                        },
                        _ => {
                            // FIXME: support variables
                            let addr = if let Some(symbols) = symbols_maybe {
                                symbols.get(&label)
                            } else {
                                None
                            };
                            // dbg!(&label, addr);
                            Instruction::AInstruction {
                                address: match addr {
                                    Some(addr) => AInstAddress::Address(*addr),
                                    _ => AInstAddress::Label(label),
                                },
                            }
                        }
                    },
                })
            }
            x if x.starts_with("(") => {
                let label = x[1..x.len() - 1].into();
                Some(Command {
                    raw: x.into(),
                    inst: Instruction::LInstruction { label },
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

    pub fn build_symbols(&self, symbols: SymbolTable) -> SymbolTable {
        let default = default_symbols();
        // Merge width default, overriding
        symbols.into_iter().chain(default).collect()
    }
}

pub fn parse(content: String) -> ParseResult {
    let parser = Parser::new(&content);
    parser.parse()
}
