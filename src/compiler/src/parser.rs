use std::iter::Peekable;

use crate::node::{
    class_var_type_from_token, item_type_from_token, sub_variant_from_token, GrammarItem,
    GrammarItemType, GrammarParamDec, GrammarSubroutineReturnType, Identifier, ParseNode,
};
use crate::token::{Keyword, Token};
use crate::tokenizer::tokenize;

type ParseError = Box<dyn std::error::Error>;
type Res<T = ()> = Result<T, ParseError>;

#[derive(Debug)]
pub struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token> + DoubleEndedIterator<Item = Token>,
{
    pub fn new<A>(tokens: A) -> Self
    where
        A: IntoIterator<IntoIter = I, Item = <I as Iterator>::Item>,
    {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse(mut self) -> Res<ParseResult> {
        let mut program_node = ParseNode::new(GrammarItem::Program);
        let class_node = self.parse_class().map_err(|x| self.parsing_error(x));
        match class_node {
            Ok(node) => {
                program_node.children.push(node);
                let result = ParseResult { root: program_node };
                Ok(result)
            }
            Err(err) => Err(format!(
                "{}\n\
            Last tokens are:\n{:?}",
                err, ""
            )
            .into()),
        }
    }

    fn parse_class(&mut self) -> Res<ParseNode> {
        self.expect(Token::Keyword(Keyword::Class))?;
        let identifier = self.parse_identifier()?;
        let mut node = ParseNode::new(GrammarItem::Class(identifier));
        let mut children: Vec<ParseNode> = vec![];
        self.expect(t::symbol("{"))?;
        children.extend(self.parse_class_var_decs()?);
        children.extend(self.parse_subroutine_decs()?);
        self.expect(t::symbol("}"))?;
        node.children.extend(children);
        Ok(node)
    }

    fn parse_class_var_decs(&mut self) -> Res<Vec<ParseNode>> {
        let mut nodes: Vec<ParseNode> = vec![];
        while let Ok(class_var_type) = expect::one_of(
            self.tokens.peek().cloned(),
            &[t::kw(Keyword::Static), t::kw(Keyword::Field)],
        ) {
            self.tokens.next();
            let (decl_type, var_names) = self.parse_var_decs_inner()?;
            nodes.push(ParseNode::new(GrammarItem::ClassVarDec(
                class_var_type_from_token(class_var_type).unwrap(),
                decl_type,
                var_names,
            )));
        }
        Ok(nodes)
    }

    fn parse_var_decs_inner(&mut self) -> Res<(GrammarItemType, Vec<Identifier>)> {
        let decl_type = match expect::something(self.tokens.next())? {
            token @ Token::Identifier(..) => item_type_from_token(token).unwrap(),
            token => item_type_from_token(expect::one_of(
                Some(token),
                &[
                    t::kw(Keyword::Int),
                    t::kw(Keyword::Boolean),
                    t::kw(Keyword::Char),
                ],
            )?)
            .unwrap(),
        };
        let mut var_names = vec![self.parse_identifier()?];
        while let Ok(..) = self.try_expect(t::symbol(",")) {
            self.tokens.next();
            var_names.push(self.parse_identifier()?);
        }
        self.expect(t::symbol(";"))?;
        Ok((decl_type, var_names))
    }

    fn parse_subroutine_decs(&mut self) -> Res<Vec<ParseNode>> {
        let mut nodes: Vec<ParseNode> = vec![];

        while let Ok(sub_variant) = expect::one_of(
            self.tokens.peek().cloned(),
            &[
                t::kw(Keyword::Constructor),
                t::kw(Keyword::Function),
                t::kw(Keyword::Method),
            ],
        ) {
            self.tokens.next();
            let return_type = match expect::something(self.tokens.next())? {
                Token::Keyword(Keyword::Void) => GrammarSubroutineReturnType::Void,
                token @ Token::Identifier(..) => {
                    GrammarSubroutineReturnType::Type(item_type_from_token(token).unwrap())
                }
                token => unreachable!("Unexpected subroutine type: {:?}", token),
            };
            let name = self.parse_identifier()?;
            self.expect(t::symbol("("))?;

            let mut params = vec![];
            while let Ok(param_token) = expect::identifier(self.tokens.peek().cloned()) {
                params.push(GrammarParamDec {
                    type_: item_type_from_token(Token::Identifier(param_token)).unwrap(),
                    ident: self.parse_identifier()?,
                });
                if self.try_expect(t::symbol(",")).is_ok() {
                    self.tokens.next();
                } else {
                    break;
                }
            }
            self.expect(t::symbol(")"))?;
            let mut node = ParseNode::new(GrammarItem::SubroutineDec(
                sub_variant_from_token(sub_variant).unwrap(),
                return_type,
                name,
                params,
            ));
            node.children = self.parse_subroutine_body()?;
            nodes.push(node);
        }
        Ok(nodes)
    }

    fn parse_subroutine_body(&mut self) -> Res<Vec<ParseNode>> {
        self.expect(t::symbol("{"))?;
        let mut var_decs: Vec<ParseNode> = vec![];
        let mut statements: Vec<ParseNode> = vec![];

        while let Ok(sub_variant) = self.try_expect(t::kw(Keyword::Var)) {
            self.tokens.next();
            let (decl_type, var_names) = self.parse_var_decs_inner()?;
            var_decs.push(ParseNode::new(GrammarItem::VarDec(decl_type, var_names)));
        }

        self.expect(t::symbol("}"))?;
        return Ok(var_decs.into_iter().chain(statements.into_iter()).collect());
    }

    fn parse_identifier(&mut self) -> Res<String> {
        Ok(expect::identifier(self.tokens.next())?)
    }

    fn expect(&mut self, token: Token) -> Res<Token> {
        expect::specific(self.tokens.next(), token)
    }

    fn try_expect(&mut self, token: Token) -> Res<Token> {
        expect::specific(self.tokens.peek().cloned(), token)
    }

    fn parsing_error(&mut self, err: ParseError) -> ParseError {
        err
        // let tok = self.tokens.next();
        // format!(
        //     "Can't parse at {1}:{2}: '{0}'\n\
        //     At token:{3:?}\n\
        //     {line}\n",
        //     err,
        //     0,
        //     0,
        //     tok,
        //     line = ""
        // )
        // .into()
    }
}

/// Utilities for reading tokens.
mod expect {
    use super::{Res, Token};

    #[must_use = "Handle the result"]
    pub fn something(token: Option<Token>) -> Res<Token> {
        if let Some(token) = token {
            Ok(token)
        } else {
            Err("expected something but got nothing".into())
        }
    }

    #[must_use = "Handle the result"]
    pub fn specific(token: Option<Token>, expected: Token) -> Res<Token> {
        let token = something(token)?;
        if token == expected {
            Ok(token)
        } else {
            Err(format!(
                "Unexpected token.\nActual: {:?}\nExpected: {:?}\n",
                token, expected
            )
            .into())
        }
    }

    #[must_use = "Handle the result"]
    pub fn identifier(token: Option<Token>) -> Res<String> {
        let token = self::something(token)?;
        if let Token::Identifier(ident) = token {
            Ok(ident)
        } else {
            Err(format!("Expected identifier, got: {:?}\n", token).into())
        }
    }

    #[must_use = "Handle the result"]
    pub fn one_of(token: Option<Token>, whitelist: &[Token]) -> Res<Token> {
        let token = self::something(token)?;
        if whitelist
            .iter()
            .any(|allowed_token| *allowed_token == token)
        {
            Ok(token)
        } else {
            Err(format!(
                "Unexpected token.\nActual: {:?}\nExpected: {:?}\n",
                token,
                whitelist.to_owned()
            )
            .into())
        }
    }
}

mod t {
    use super::{Keyword, Token};

    pub fn symbol(s: &str) -> Token {
        Token::Symbol(s.into())
    }

    pub fn kw(kw: Keyword) -> Token {
        Token::Keyword(kw)
    }
}

#[derive(Debug)]
pub struct ParseResult {
    root: ParseNode,
}

pub fn parse(input: &str) -> Result<ParseResult, Box<dyn std::error::Error>> {
    let parser = Parser::new(tokenize(input)?);
    parser.parse()
}

pub fn result_to_xml(result: ParseResult) -> String {
    result.root.to_xml()
}
