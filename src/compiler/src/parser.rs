use std::iter::{Enumerate, Peekable};

use crate::{
    node::*,
    token::{Keyword, Token},
    tokenizer::tokenize,
};

type ParseError = Box<dyn std::error::Error>;
type Res<T = ()> = Result<T, ParseError>;

#[derive(Debug)]
pub struct Parser<I>
where
    I: Iterator<Item = Token> + DoubleEndedIterator<Item = Token> + ExactSizeIterator<Item = Token>,
{
    tokens: Peekable<Enumerate<I>>,
    pos: usize,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token> + DoubleEndedIterator<Item = Token> + ExactSizeIterator<Item = Token>,
{
    pub fn new<A>(tokens: A) -> Self
    where
        A: IntoIterator<IntoIter = I, Item = <I as Iterator>::Item>,
    {
        Self {
            tokens: tokens.into_iter().enumerate().peekable(),
            pos: 0,
        }
    }

    pub fn parse(mut self) -> Res<ParseResult> {
        let class_node = self.parse_class().map_err(|x| self.parsing_error(x))?;
        let result = ParseResult { root: class_node };
        Ok(result)
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
            self.peek(),
            &[t::kw(Keyword::Static), t::kw(Keyword::Field)],
        ) {
            self.next();
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
        let decl_type = match expect::something(self.next())? {
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
            self.next();
            var_names.push(self.parse_identifier()?);
        }
        self.expect(t::symbol(";"))?;
        Ok((decl_type, var_names))
    }

    fn parse_subroutine_decs(&mut self) -> Res<Vec<SubroutineDec>> {
        let mut nodes: Vec<SubroutineDec> = vec![];

        while let Ok(sub_variant) = expect::one_of(
            self.peek(),
            &[
                t::kw(Keyword::Constructor),
                t::kw(Keyword::Function),
                t::kw(Keyword::Method),
            ],
        ) {
            self.next();
            let return_type = match expect::something(self.next())? {
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
        while let Ok(param_token) = expect::something(self.peek()) {
            if param_token == t::symbol(")") {
                break;
            }
            let type_ = item_type_from_token(param_token).unwrap();
            self.next();
            let ident = self.parse_identifier()?;
            params.push(GrammarParamDec { type_, ident });
            if self.try_expect(t::symbol(",")).is_ok() {
                self.next();
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
            self.next();
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
            self.peek(),
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
        Ok(match expect::something(self.peek())? {
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
                self.next();
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
                self.next();
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
        let call = self.parse_subroutine_call(None)?;
        self.expect(t::symbol(";"))?;
        Ok(Statement::DoStatement(DoStatement { call }))
    }

    fn parse_statement_return(&mut self) -> Res<Statement> {
        self.expect(t::kw(Keyword::Return))?;
        let result = match expect::something(self.peek())? {
            Token::Symbol(s) if s == ";" => None,
            _ => Some(self.parse_expression()?),
        };
        self.expect(t::symbol(";"))?;
        Ok(Statement::ReturnStatement(ReturnStatement { result }))
    }

    fn parse_expression(&mut self) -> Res<Expr> {
        let term = self.parse_term()?;
        let mut terms: Vec<(Op, Term)> = vec![];
        if let Some(op) = expect::something(self.peek())?.get_op() {
            self.next();
            terms.push((Op(op), self.parse_term()?));
        };
        Ok(Expr(term, terms))
    }

    fn parse_term(&mut self) -> Res<Term> {
        Ok(match expect::something(self.peek())? {
            Token::IntegerConst(i) => {
                self.next();
                Term::IntegerConstant(i)
            }
            Token::StringConst(s) => {
                self.next();
                Term::StringConst(s)
            }
            Token::Keyword(kw)
                if [Keyword::True, Keyword::False, Keyword::This, Keyword::Null].contains(&kw) =>
            {
                self.next();
                Term::KeywordConstant(kw)
            }
            Token::Symbol(s) if s == "(" => {
                self.next();
                let expr = self.parse_expression()?;
                self.expect(t::symbol(")"))?;
                Term::ParenExpr(Box::new(expr))
            }
            Token::Symbol(op) => {
                if !Token::is_unary_op(op.as_str()) {
                    return Err("Invalid unary op")?;
                }
                self.next();
                Term::UnaryOp(op, Box::new(self.parse_term()?))
            }
            Token::Identifier(ident) => {
                self.next();
                match expect::something(self.peek())? {
                    Token::Symbol(s) if s == "[" => {
                        self.next();
                        let expr = self.parse_expression()?;
                        self.expect(t::symbol("]"))?;
                        Term::IndexExpr(ident, Box::new(expr))
                    }
                    Token::Symbol(s) if s == "." || s == "(" => {
                        Term::SubroutineCall(self.parse_subroutine_call(Some(ident))?)
                    }
                    _ => Term::VarName(ident),
                }
            }
            _token => todo!("Unsupported term: {:?}", _token),
        })
    }

    fn parse_expression_list(&mut self) -> Res<ExprList> {
        let mut list = vec![];
        while let Ok(param_token) = expect::something(self.peek()) {
            if param_token == t::symbol(")") {
                break;
            }
            list.push(self.parse_expression()?);
            if self.try_expect(t::symbol(",")).is_ok() {
                self.next();
            } else {
                break;
            }
        }
        Ok(list)
    }

    fn parse_subroutine_call(&mut self, maybe_name: Option<String>) -> Res<SubroutineCall> {
        let name: String = match maybe_name {
            Some(s) => s,
            None => self.parse_identifier()?,
        };
        match expect::something(self.peek())? {
            Token::Symbol(x) if x == "." => {
                self.next();
                let method_name = self.parse_identifier()?;
                self.expect(t::symbol("("))?;
                let expr_list = self.parse_expression_list()?;
                self.expect(t::symbol(")"))?;
                Ok(SubroutineCall::MethodCall(name, method_name, expr_list))
            }
            Token::Symbol(x) if x == "(" => {
                self.next();
                let expr_list = self.parse_expression_list()?;
                self.expect(t::symbol(")"))?;
                Ok(SubroutineCall::SimpleCall(name, expr_list))
            }
            _ => Err("Can't parse subroutine call")?,
        }
    }

    fn parse_identifier(&mut self) -> Res<String> {
        // dbg!(self.tokens.peek());
        Ok(expect::identifier(self.next())?)
    }

    fn next(&mut self) -> Option<Token> {
        self.pos += 1;
        self.tokens.next().map(|x| x.1) //.map(|x| dbg!(x))
    }

    fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().cloned().map(|x| x.1) //.map(|x| dbg!(x))
    }

    fn expect(&mut self, token: Token) -> Res<Token> {
        expect::specific(self.next(), token)
    }

    fn try_expect(&mut self, token: Token) -> Res<Token> {
        expect::specific(self.peek(), token)
    }

    fn parsing_error(mut self, err: ParseError) -> ParseError {
        let _tok = self.next();
        // let last_n_size = 10;
        // let pos = self.pos;
        let last_tokens: Vec<Token> = vec![];
        // let last_tokens: Vec<_> = self
        //     .tokens
        //     .rev()
        //     // .skip(pos)
        //     // .rev()
        //     // .take(last_n_size)
        //     // .map(|x| x.1)
        //     .collect();
        format!(
            "{}\n\
            Last tokens:\n{}",
            err,
            last_tokens
                .into_iter()
                .enumerate()
                .map(|(i, t)| format!("{}: {:?}", i, t))
                .collect::<Vec<_>>()
                .join("\n")
        )
        .into()
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
    pub root: Class,
}

pub fn parse(input: &str) -> Result<ParseResult, Box<dyn std::error::Error>> {
    let parser = Parser::new(tokenize(input)?);
    parser.parse()
}
