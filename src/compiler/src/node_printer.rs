use std::fmt::Write;

use crate::{node::*, token::keyword_to_string, xml::xml_wrap_declaration as xwd};

#[derive(Debug, Clone)]
pub enum Node {
    Class(Class),
    ClassVarDec(ClassVarDec),
    VarDec(VarDec),
    SubroutineDec(SubroutineDec),
    ParameterList(Vec<GrammarParamDec>),
    SubroutineBody(Subroutine),
    Statements(Vec<Statement>),
    Statement(Statement),
    Expr(Expr),
    ParenExpr(Expr),
    Term(Term),
    ExprList(Vec<Expr>),
    SubroutineCall(SubroutineCall),
}

const XML_LEVEL_INDENT: usize = 2;

fn wi<S>(out: &mut dyn Write, s: S, indent: usize)
where
    S: std::fmt::Display,
{
    write!(out, "{:indent$}{}\n", "", s, indent = indent).unwrap();
}

pub fn print_to_xml(out: &mut dyn Write, node: Node, indent_: Option<usize>) {
    let indent = indent_.unwrap_or(0);
    let body_indent = indent + XML_LEVEL_INDENT;

    macro_rules! w(
        ($s:expr) => (
            wi(out, $s, body_indent)
        );
        ($s:expr, $indent:expr) => (
            wi(out, $s, indent)
        );
    );
    macro_rules! print_child(
        ($s:expr) => (
            print_to_xml(out, $s, Some(body_indent))
        );
        ($s:expr, $i:expr) => (
            print_to_xml(out, $s, Some($i))
        );
    );

    match node {
        Node::Class(Class(ident, var_dec, sub_dec)) => {
            w!("<class>", indent);
            w!(xwd("keyword", "class"));
            w!(xwd("identifier", ident.as_str()));
            w!(xwd("symbol", "{"));

            var_dec
                .into_iter()
                .for_each(|x| print_child!(Node::ClassVarDec(x)));
            sub_dec
                .into_iter()
                .for_each(|x| print_child!(Node::SubroutineDec(x)));

            w!(xwd("symbol", "}"));
            w!("</class>", indent);
        }
        Node::SubroutineDec(SubroutineDec(variant, type_, ident, params, sub)) => {
            w!("<subroutineDec>", indent);
            w!(xwd(
                "keyword",
                match variant {
                    GrammarSubroutineVariant::Constructor => "constructor",
                    GrammarSubroutineVariant::Function => "function",
                    GrammarSubroutineVariant::Method => "method",
                },
            ));
            w!(match type_ {
                GrammarSubroutineReturnType::Void => xwd("keyword", "void"),
                GrammarSubroutineReturnType::Type(t) => print_type_to_xml(t),
            });
            w!(xwd("identifier", ident.as_str()));

            w!(xwd("symbol", "("));
            print_child!(Node::ParameterList(params));
            w!(xwd("symbol", ")"));
            print_child!(Node::SubroutineBody(sub));
            w!("</subroutineDec>", indent);
        }
        Node::ParameterList(params_) => {
            w!("<parameterList>", indent);
            let mut params = params_.into_iter();
            if let Some(param) = params.next() {
                w!(print_type_to_xml(param.type_));
                w!(xwd("identifier", param.ident.as_str()));
                for param in params {
                    w!(xwd("symbol", ","));
                    w!(print_type_to_xml(param.type_));
                    w!(xwd("identifier", param.ident.as_str()));
                }
            }
            w!("</parameterList>", indent);
        }
        Node::SubroutineBody(Subroutine(var_dec, statements)) => {
            w!("<subroutineBody>", indent);
            w!(xwd("symbol", "{"));
            var_dec
                .into_iter()
                .for_each(|x| print_child!(Node::VarDec(x)));
            print_child!(Node::Statements(statements));
            w!(xwd("symbol", "}"));
            w!("</subroutineBody>", indent);
        }
        Node::ClassVarDec(ClassVarDec(class_var_type, item_type, names)) => {
            w!("<classVarDec>", indent);
            w!(xwd(
                "keyword",
                match class_var_type {
                    GrammarClassVarType::Field => "field",
                    GrammarClassVarType::Static => "static",
                }
            ));
            w!(print_type_to_xml(item_type));
            let mut names_iter = names.iter();
            if let Some(name) = names_iter.next() {
                w!(xwd("identifier", name));
                for name in names_iter {
                    w!(xwd("symbol", ","));
                    w!(xwd("identifier", name));
                }
            }
            w!(xwd("symbol", ";"));
            w!("</classVarDec>", indent);
        }
        Node::VarDec(VarDec(type_, names)) => {
            w!("<varDec>", indent);
            w!(xwd("keyword", "var"));
            w!(print_type_to_xml(type_));
            let mut names_iter = names.iter();
            if let Some(name) = names_iter.next() {
                w!(xwd("identifier", name));
                for name in names_iter {
                    w!(xwd("symbol", ","));
                    w!(xwd("identifier", name));
                }
            }
            w!(xwd("symbol", ";"));
            w!("</varDec>", indent);
        }
        Node::Statements(statements) => {
            w!("<statements>", indent);
            statements
                .into_iter()
                .for_each(|x| print_child!(Node::Statement(x)));
            w!("</statements>", indent);
        }
        Node::Statement(Statement::ReturnStatement(ReturnStatement { result })) => {
            w!("<returnStatement>", indent);
            w!(xwd("keyword", "return"));
            if let Some(expr) = result {
                print_child!(Node::Expr(expr));
            }
            w!(xwd("symbol", ";"));
            w!("</returnStatement>", indent);
        }
        Node::Statement(Statement::LetStatement(LetStatement {
            index_expr,
            name,
            value_expr,
        })) => {
            w!("<letStatement>", indent);
            w!(xwd("keyword", "let"));
            w!(xwd("identifier", name.as_str()));
            if let Some(expr) = index_expr {
                w!(xwd("symbol", "["));
                print_child!(Node::Expr(expr));
                w!(xwd("symbol", "]"));
            }
            w!(xwd("symbol", "="));
            print_child!(Node::Expr(value_expr));
            w!(xwd("symbol", ";"));
            w!("</letStatement>", indent);
        }
        Node::Statement(Statement::IfStatement(IfStatement {
            if_expr,
            if_statements,
            else_statements,
        })) => {
            w!("<ifStatement>", indent);
            w!(xwd("keyword", "if"));
            w!(xwd("symbol", "("));
            print_child!(Node::Expr(if_expr));
            w!(xwd("symbol", ")"));
            w!(xwd("symbol", "{"));
            print_child!(Node::Statements(if_statements));
            w!(xwd("symbol", "}"));
            if let Some(statements) = else_statements {
                w!(xwd("keyword", "else"));
                w!(xwd("symbol", "{"));
                print_child!(Node::Statements(statements));
                w!(xwd("symbol", "}"));
            }
            w!("</ifStatement>", indent);
        }
        Node::Statement(Statement::WhileStatement(WhileStatement {
            cond_expr,
            statements,
        })) => {
            w!("<whileStatement>", indent);
            w!(xwd("keyword", "while"));
            w!(xwd("symbol", "("));
            print_child!(Node::Expr(cond_expr));
            w!(xwd("symbol", ")"));
            w!(xwd("symbol", "{"));
            print_child!(Node::Statements(statements));
            w!(xwd("symbol", "}"));
            w!("</whileStatement>", indent);
        }
        Node::Statement(Statement::DoStatement(DoStatement { call })) => {
            w!("<doStatement>", indent);
            w!(xwd("keyword", "do"));
            print_child!(Node::SubroutineCall(call), indent);
            w!(xwd("symbol", ";"));
            w!("</doStatement>", indent);
        }
        Node::SubroutineCall(call) => {
            let (this_ident, method, args) = match call {
                SubroutineCall::SimpleCall(method, args) => (None, method, args),
                SubroutineCall::MethodCall(this_ident, method, args) => {
                    (Some(this_ident), method, args)
                }
            };
            if let Some(ident) = this_ident {
                w!(xwd("identifier", ident.as_str()));
                w!(xwd("symbol", "."));
            }
            w!(xwd("identifier", method.as_str()));
            w!(xwd("symbol", "("));
            print_child!(Node::ExprList(args));
            w!(xwd("symbol", ")"));
        }
        Node::Expr(Expr(term, terms)) => {
            w!("<expression>", indent);
            print_child!(Node::Term(term));
            for (Op(op), term) in terms {
                w!(xwd("symbol", op.as_str()));
                print_child!(Node::Term(term));
            }
            w!("</expression>", indent);
        }
        Node::Term(term) => {
            w!("<term>", indent);
            match term {
                Term::VarName(ident) => w!(xwd("identifier", ident.as_str())),
                Term::KeywordConstant(kw) => w!(xwd("keyword", keyword_to_string(&kw))),
                Term::IntegerConstant(i) => w!(xwd("integerConstant", i.to_string().as_str())),
                Term::StringConst(s) => w!(xwd("stringConstant", s.as_str())),
                Term::UnaryOp(op, term) => {
                    w!(xwd("symbol", op.as_str()));
                    print_child!(Node::Term(*term));
                }
                Term::ParenExpr(expr) => {
                    print_child!(Node::ParenExpr(*expr), indent);
                }
                Term::IndexExpr(ident, expr) => {
                    w!(xwd("identifier", ident.as_str()));
                    w!(xwd("symbol", "["));
                    print_child!(Node::Expr(*expr));
                    w!(xwd("symbol", "]"));
                }
                Term::SubroutineCall(call) => {
                    print_child!(Node::SubroutineCall(call), indent);
                }
            };
            w!("</term>", indent);
        }
        Node::ParenExpr(expr) => {
            w!(xwd("symbol", "("));
            print_child!(Node::Expr(expr));
            w!(xwd("symbol", ")"));
        }
        Node::ExprList(exprs_) => {
            w!("<expressionList>", indent);
            let mut exprs = exprs_.into_iter();
            if let Some(expr) = exprs.next() {
                print_child!(Node::Expr(expr));
                for expr in exprs {
                    w!(xwd("symbol", ","));
                    print_child!(Node::Expr(expr))
                }
            }
            w!("</expressionList>", indent);
        }
    }
}

fn print_type_to_xml(type_: GrammarItemType) -> String {
    match type_ {
        GrammarItemType::Int => xwd("keyword", "int"),
        GrammarItemType::Char => xwd("keyword", "char"),
        GrammarItemType::Boolean => xwd("keyword", "boolean"),
        GrammarItemType::Class(ident) => xwd("identifier", ident.as_str()),
    }
}
