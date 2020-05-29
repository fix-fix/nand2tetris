macro_rules! enum_str {
    ($q:ident enum $name:ident {
        $($variant:ident = $val:expr),*,
    }) => {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        $q enum $name {
            $($variant = $val),*
        }

        impl $name {
            fn name(&self) -> u8 {
                match self {
                    $($name::$variant => $val),*
                }
            }
        }
    };
}

enum_str! {
pub enum Dest {
    Null= 0b000,
    M= 0b001,
    D= 0b010,
    MD= 0b011,
    A= 0b100,
    AM= 0b110,
    AD= 0b101,
    AMD= 0b111,
}}

impl std::fmt::Binary for Dest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self.name(), f)
    }
}

enum_str! {
pub enum Jump {
    Null = 0b000,
    JGT = 0b001,
    JEQ = 0b010,
    JGE = 0b011,
    JLT = 0b100,
    JNE = 0b110,
    JLE = 0b101,
    JMP = 0b111,
}}

impl std::fmt::Binary for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self.name(), f)
    }
}

#[derive(Debug)]
pub struct Comp(String);

impl Comp {
    pub fn fmt_binary(&self) -> &'static str {
        let Self(s) = self;
        match s as &str {
            "0" => "0101010",
            "1" => "0111111",
            "-1" => "0111010",

            "D" => "0001100",
            "A" => "0110000",
            "!D" => "0001111",
            "!A" => "0110011",
            "-D" => "0001111",
            "-A" => "0110011",
            "D+1" => "0011111",
            "A+1" => "0110111",
            "D-1" => "0001111",
            "A-1" => "0110010",
            "D+A" => "0000010",
            "D-A" => "0010011",
            "A-D" => "0000111",
            "D&A" => "0000000",
            "D|A" => "0010101",

            "M" => "1110000",
            "!M" => "1110011",
            "-M" => "1110011",
            "M+1" => "1110111",
            "M-1" => "1110010",
            "D+M" => "1000010",
            "D-M" => "1010011",
            "M-D" => "1000111",
            "D&M" => "1000000",
            "D|M" => "1010101",

            _ => "0000000",
        }
    }
}

pub fn make_comp(s: String) -> Comp {
    Comp(s)
}

#[derive(Debug)]
pub enum Instruction {
    AInstruction { address: i32 },
    CInstruction { comp: Comp, dest: Dest, jump: Jump },
    LInstruction { label_address: i32 },
}
