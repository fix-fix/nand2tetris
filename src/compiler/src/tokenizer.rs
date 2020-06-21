use crate::token::*;

pub fn tokenize(_input: String) -> Vec<Token> {
    vec![Token::Symbol('=')]
}

pub fn tokens_to_xml(_tokens: Vec<Token>) -> String {
    format!("{:?}", _tokens)
}
