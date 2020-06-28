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
pub struct Type(pub GrammarItemType);

#[derive(Debug, Clone)]
pub struct Class(pub Identifier, pub Vec<ClassVarDec>, pub Vec<SubroutineDec>);

#[derive(Debug, Clone)]
pub struct ClassVarDec(
    pub GrammarClassVarType,
    pub GrammarItemType,
    pub Vec<Identifier>,
);

#[derive(Debug, Clone)]
pub struct VarDec(pub GrammarItemType, pub Vec<Identifier>);

#[derive(Debug, Clone)]
pub struct SubroutineDec(
    pub GrammarSubroutineVariant,
    pub GrammarSubroutineReturnType,
    pub Identifier,
    pub Vec<GrammarParamDec>,
    pub Subroutine,
);

#[derive(Debug, Clone)]
pub struct Subroutine(pub Vec<VarDec>, pub Vec<Statement>);

#[derive(Debug, Clone)]
pub enum Statement {
    LetStatement(LetStatement),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    DoStatement(DoStatement),
    ReturnStatement(ReturnStatement),
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: Identifier,
    pub index_expr: Option<Expr>,
    pub value_expr: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub if_expr: Expr,
    pub if_statements: Vec<Statement>,
    pub else_statements: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub cond_expr: Expr,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct DoStatement {
    pub call: SubroutineCall,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub result: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Expr(pub Term, pub Vec<(Op, Term)>);

pub type ExprList = Vec<Expr>;

#[derive(Debug, Clone)]
pub enum Term {
    VarName(Identifier),
    KeywordConstant(Keyword),
}

#[derive(Debug, Clone)]
pub struct Op(pub String);

#[derive(Debug, Clone)]
pub enum SubroutineCall {
    SimpleCall(Identifier, ExprList),
    MethodCall(Identifier, Identifier, ExprList),
}

impl Class {
    pub fn to_xml(&self) -> String {
        format!("{:#?}", &self)
    }
}
