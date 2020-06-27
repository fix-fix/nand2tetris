use crate::token::{Keyword, Token};

pub type Identifier = String;

#[derive(Debug, Clone)]
pub enum GrammarItemType {
    Int,
    Char,
    Boolean,
    Class(Identifier),
}

pub fn item_type_from_token(t: Token) -> Option<GrammarItemType> {
    Some(match t {
        Token::Keyword(kw) => match kw {
            Keyword::Int => GrammarItemType::Int,
            Keyword::Char => GrammarItemType::Char,
            Keyword::Boolean => GrammarItemType::Boolean,
            _ => return None,
        },
        Token::Identifier(ident) => GrammarItemType::Class(ident),
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub enum GrammarClassVarType {
    Static,
    Field,
}

pub fn class_var_type_from_token(t: Token) -> Option<GrammarClassVarType> {
    Some(match t {
        Token::Keyword(Keyword::Static) => GrammarClassVarType::Static,
        Token::Keyword(Keyword::Field) => GrammarClassVarType::Field,
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub enum GrammarSubroutineReturnType {
    Void,
    Type(GrammarItemType),
}

#[derive(Debug, Clone)]
pub enum GrammarSubroutineVariant {
    Constructor,
    Function,
    Method,
}

pub fn sub_variant_from_token(t: Token) -> Option<GrammarSubroutineVariant> {
    Some(match t {
        Token::Keyword(Keyword::Constructor) => GrammarSubroutineVariant::Constructor,
        Token::Keyword(Keyword::Function) => GrammarSubroutineVariant::Function,
        Token::Keyword(Keyword::Method) => GrammarSubroutineVariant::Method,
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub struct GrammarParamDec {
    pub type_: GrammarItemType,
    pub ident: Identifier,
}

#[derive(Debug, Clone)]
pub enum GrammarItem {
    Program,
    Class(Identifier),
    ClassVarDec(GrammarClassVarType, GrammarItemType, Vec<Identifier>),
    VarDec(GrammarItemType, Vec<Identifier>),
    SubroutineDec(GrammarSubroutineVariant, GrammarSubroutineReturnType, Identifier, Vec<GrammarParamDec>),
    Type(GrammarItemType),
}

#[derive(Debug, Clone)]
pub struct ParseNode {
    pub children: Vec<ParseNode>,
    pub entry: GrammarItem,
}

impl ParseNode {
    pub fn new(item: GrammarItem) -> Self {
        Self {
            children: vec![],
            entry: item,
        }
    }
    pub fn to_xml(&self) -> String {
        format!("{:#?}", &self)
    }
}
