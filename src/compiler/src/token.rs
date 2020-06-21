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
    Symbol(char),
    // TODO:
}
