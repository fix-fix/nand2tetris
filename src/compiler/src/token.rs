use crate::xml::*;

// #[derive(Debug)]
// pub enum TokenKind {
//     Keyword,
//     Symbol,
//     Identifier,
//     IntegerConst,
//     StringConst,
// }

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Symbol(String),
    Identifier(String),
    IntegerConst(u16),
    StringConst(String),
}

impl Token {
    pub fn as_xml_decl(&self) -> String {
        match self {
            Self::Keyword(keyword) => xml_wrap_declaration("keyword", keyword_to_string(keyword)),
            Self::Symbol(symbol) => xml_wrap_declaration("symbol", symbol),
            Self::Identifier(identifier) => xml_wrap_declaration("identifier", identifier),
            Self::IntegerConst(num) => xml_wrap_declaration("integerConstant", &num.to_string()),
            Self::StringConst(s) => xml_wrap_declaration("stringConstant", s),
        }
    }
    pub fn get_op(&self) -> Option<String> {
        match self {
            Self::Symbol(symbol) => {
                if ["+", "-", "*", "/", "&", "|", "<", ">", "="].contains(&symbol.as_str()) {
                    Some(symbol.into())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub const KEYWORDS: &[&str] = &[
    "class",
    "constructor",
    "function",
    "method",
    "field",
    "static",
    "var",
    "int",
    "char",
    "boolean",
    "void",
    "true",
    "false",
    "null",
    "this",
    "let",
    "do",
    "if",
    "else",
    "while",
    "return",
];

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

pub fn keyword_from_string(s: &str) -> Option<Keyword> {
    Some(match s {
        "class" => Keyword::Class,
        "constructor" => Keyword::Constructor,
        "function" => Keyword::Function,
        "method" => Keyword::Method,
        "field" => Keyword::Field,
        "static" => Keyword::Static,
        "var" => Keyword::Var,
        "int" => Keyword::Int,
        "char" => Keyword::Char,
        "boolean" => Keyword::Boolean,
        "void" => Keyword::Void,
        "true" => Keyword::True,
        "false" => Keyword::False,
        "null" => Keyword::Null,
        "this" => Keyword::This,
        "let" => Keyword::Let,
        "do" => Keyword::Do,
        "if" => Keyword::If,
        "else" => Keyword::Else,
        "while" => Keyword::While,
        "return" => Keyword::Return,
        _ => return None,
    })
}

fn keyword_to_string(keyword: &Keyword) -> &'static str {
    match keyword {
        Keyword::Class => "class",
        Keyword::Constructor => "constructor",
        Keyword::Function => "function",
        Keyword::Method => "method",
        Keyword::Field => "field",
        Keyword::Static => "static",
        Keyword::Var => "var",
        Keyword::Int => "int",
        Keyword::Char => "char",
        Keyword::Boolean => "boolean",
        Keyword::Void => "void",
        Keyword::True => "true",
        Keyword::False => "false",
        Keyword::Null => "null",
        Keyword::This => "this",
        Keyword::Let => "let",
        Keyword::Do => "do",
        Keyword::If => "if",
        Keyword::Else => "else",
        Keyword::While => "while",
        Keyword::Return => "return",
    }
}
