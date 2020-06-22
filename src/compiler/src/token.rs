use crate::xml::*;

#[derive(Debug)]
pub enum TokenType {
    Keyword,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

#[derive(Debug)]
pub enum Token {
    Symbol(String),
    // TODO:
}

impl Token {
    pub fn as_xml_decl(&self) -> String {
        match self {
            Self::Symbol(symbol) => xml_wrap_declaration("symbol".into(), symbol.into()),
        }
    }
}
