use std::iter::Peekable;

use crate::node::*;
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
        let class_node = self.parse_class().map_err(|x| self.parsing_error(x));
        match class_node {
            Ok(node) => {
                let result = ParseResult { root: node };
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

    fn parse_class(&mut self) -> Res<Class> {
        self.expect(Token::Keyword(Keyword::Class))?;
        let identifier = self.parse_identifier()?;
        self.expect(t::symbol("{"))?;
        let var_decs = self.parse_class_var_decs()?;
        let sub_decs = self.parse_subroutine_decs()?;
        self.expect(t::symbol("}"))?;
        Ok(Class(identifier, var_decs, sub_decs))
    }

    fn parse_class_var_decs(&mut self) -> Res<Vec<ClassVarDec>> {
        let mut nodes: Vec<ClassVarDec> = vec![];
        while let Ok(class_var_type) = expect::one_of(
            self.tokens.peek().cloned(),
            &[t::kw(Keyword::Static), t::kw(Keyword::Field)],
        ) {
            self.tokens.next();
            let (decl_type, var_names) = self.parse_var_decs_inner()?;
            nodes.push(ClassVarDec(
                class_var_type_from_token(class_var_type).unwrap(),
                decl_type,
                var_names,
            ));
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

    fn parse_subroutine_decs(&mut self) -> Res<Vec<SubroutineDec>> {
        let mut nodes: Vec<SubroutineDec> = vec![];

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
            let params = self.parse_parameters_list()?;
            self.expect(t::symbol(")"))?;

            let body = self.parse_subroutine_body()?;
            let node = SubroutineDec(
                sub_variant_from_token(sub_variant).unwrap(),
                return_type,
                name,
                params,
                body,
            );
            nodes.push(node);
        }
        Ok(nodes)
    }

    fn parse_parameters_list(&mut self) -> Res<Vec<GrammarParamDec>> {
        let mut params = vec![];
        while let Ok(param_token) = expect::something(self.tokens.peek().cloned()) {
            if param_token == t::symbol(")") {
                break;
            }
            let type_ = item_type_from_token(param_token).unwrap();
            self.tokens.next();
            let ident = self.parse_identifier()?;
            params.push(GrammarParamDec { type_, ident });
            if self.try_expect(t::symbol(",")).is_ok() {
                self.tokens.next();
            } else {
                break;
            }
        }
        return Ok(params);
    }

    fn parse_subroutine_body(&mut self) -> Res<Subroutine> {
        self.expect(t::symbol("{"))?;
        let mut var_decs: Vec<VarDec> = vec![];
        while let Ok(..) = self.try_expect(t::kw(Keyword::Var)) {
            self.tokens.next();
            let (decl_type, var_names) = self.parse_var_decs_inner()?;
            var_decs.push(VarDec(decl_type, var_names));
        }

        let statements = self.parse_statements()?;

        self.expect(t::symbol("}"))?;
        return Ok(Subroutine(var_decs, statements));
    }

    fn parse_statements(&mut self) -> Res<Vec<Statement>> {
        let mut statements: Vec<Statement> = vec![];
        while let Ok(..) = expect::one_of(
            self.tokens.peek().cloned(),
            &[
                t::kw(Keyword::Let),
                t::kw(Keyword::If),
                t::kw(Keyword::While),
                t::kw(Keyword::Do),
                t::kw(Keyword::Return),
            ],
        ) {
            statements.push(self.parse_statement()?);
        }
        return Ok(statements);
    }

    fn parse_statement(&mut self) -> Res<Statement> {
        Ok(match expect::something(self.tokens.peek().cloned())? {
            Token::Keyword(Keyword::Let) => self.parse_statement_let()?,
            Token::Keyword(Keyword::If) => self.parse_statement_if()?,
            Token::Keyword(Keyword::While) => self.parse_statement_while()?,
            Token::Keyword(Keyword::Do) => self.parse_statement_do()?,
            Token::Keyword(Keyword::Return) => self.parse_statement_return()?,
            statement_token => {
                return Err(
                    format!("Unexpected statement token type: {:?}", statement_token).into(),
                )
            }
        })
    }

    fn parse_statement_let(&mut self) -> Res<Statement> {
        self.expect(t::kw(Keyword::Let))?;
        let name = self.parse_identifier()?;
        let index_expr = match self.try_expect(t::symbol("[")) {
            Ok(..) => {
                self.tokens.next();
                let expr = self.parse_expression()?;
                self.expect(t::symbol("]"))?;
                Some(expr)
            }
            _ => None,
        };
        self.expect(t::symbol("="))?;
        let value_expr = self.parse_expression()?;
        self.expect(t::symbol(";"))?;
        Ok(Statement::LetStatement(LetStatement {
            name,
            index_expr,
            value_expr,
        }))
    }

    fn parse_statement_if(&mut self) -> Res<Statement> {
        self.expect(t::kw(Keyword::If))?;
        self.expect(t::symbol("("))?;
        let if_expr = self.parse_expression()?;
        self.expect(t::symbol(")"))?;
        self.expect(t::symbol("{"))?;
        let if_statements = self.parse_statements()?;
        self.expect(t::symbol("}"))?;
        let else_statements = match self.try_expect(t::kw(Keyword::Else)) {
            Ok(..) => {
                self.tokens.next();
                self.expect(t::symbol("{"))?;
                let statements = self.parse_statements()?;
                self.expect(t::symbol("}"))?;
                Some(statements)
            }
            _ => None,
        };
        Ok(Statement::IfStatement(IfStatement {
            if_expr,
            if_statements,
            else_statements,
        }))
    }

    fn parse_statement_while(&mut self) -> Res<Statement> {
        self.expect(t::kw(Keyword::While))?;
        self.expect(t::symbol("("))?;
        let cond_expr = self.parse_expression()?;
        self.expect(t::symbol(")"))?;
        self.expect(t::symbol("{"))?;
        let statements = self.parse_statements()?;
        self.expect(t::symbol("}"))?;
        Ok(Statement::WhileStatement(WhileStatement {
            cond_expr,
            statements,
        }))
    }

    fn parse_statement_do(&mut self) -> Res<Statement> {
        self.expect(t::kw(Keyword::Do))?;
        let name = self.parse_identifier()?;
        let call = match expect::something(self.tokens.peek().cloned())? {
            Token::Symbol(x) if x == "." => {
                self.tokens.next();
                let method_name = self.parse_identifier()?;
                self.expect(t::symbol("("))?;
                let expr_list = self.parse_expression_list()?;
                self.expect(t::symbol(")"))?;
                Ok(SubroutineCall::MethodCall(name, method_name, expr_list))
            }
            Token::Symbol(x) if x == "(" => {
                self.tokens.next();
                let expr_list = self.parse_expression_list()?;
                self.expect(t::symbol(")"))?;
                Ok(SubroutineCall::SimpleCall(name, expr_list))
            }
            _ => Err("Can't parse subroutine call").into(),
        }?;
        self.expect(t::symbol(";"))?;
        Ok(Statement::DoStatement(DoStatement { call }))
    }

    fn parse_statement_return(&mut self) -> Res<Statement> {
        self.expect(t::kw(Keyword::Return))?;
        let result = match expect::something(self.tokens.peek().cloned())? {
            Token::Symbol(s) if s == ";" => None,
            _ => Some(self.parse_expression()?),
        };
        self.expect(t::symbol(";"))?;
        Ok(Statement::ReturnStatement(ReturnStatement { result }))
    }

    fn parse_expression(&mut self) -> Res<Expr> {
        // TODO: Add expr parsing
        let term = self.parse_term()?;
        let mut terms: Vec<(Op, Term)> = vec![];
        if let Some(op) = expect::something(self.tokens.peek().cloned())?.get_op() {
            self.tokens.next();
            terms.push((Op(op), self.parse_term()?));
        };
        Ok(Expr(term, terms))
    }

    fn parse_term(&mut self) -> Res<Term> {
        // TODO: Add expr parsing
        Ok(match expect::something(self.tokens.peek().cloned())? {
            Token::Identifier(..) => Term::VarName(self.parse_identifier()?),
            Token::Keyword(kw) => {
                self.tokens.next();
                Term::KeywordConstant(kw)
            }
            _token => todo!("Unsupported term: {:?}", _token),
        })
    }

    fn parse_expression_list(&mut self) -> Res<ExprList> {
        // TODO: Add expr parsing
        let mut list = vec![];
        while let Ok(param_token) = expect::something(self.tokens.peek().cloned()) {
            if param_token == t::symbol(")") {
                break;
            }
            list.push(self.parse_expression()?);
            if self.try_expect(t::symbol(",")).is_ok() {
                self.tokens.next();
            } else {
                break;
            }
        }
        Ok(list)
    }

    fn parse_identifier(&mut self) -> Res<String> {
        // dbg!(self.tokens.peek());
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
    root: Class,
}

pub fn parse(input: &str) -> Result<ParseResult, Box<dyn std::error::Error>> {
    let parser = Parser::new(tokenize(input)?);
    parser.parse()
}

pub fn result_to_xml(result: ParseResult) -> String {
    result.root.to_xml()
}
