use crate::compiler::CompilerState;

use crate::codegen::{write_arith, write_push, WriteInst};
use crate::node::{Expr, Op, Term};
use crate::symbol_table::SymbolTable;
use crate::token::Keyword;

pub fn optimize_syntax_tree_expression(
    _state: &mut CompilerState,
    Expr(term1, ops): &Expr,
) -> Option<Vec<WriteInst>> {
    optimize_expr(None, term1, ops)
}

fn optimize_expr_int_inner(
    _sym_table: Option<&SymbolTable>,
    term1: &Term,
    ops: &[(Op, Term)],
) -> Option<i16> {
    // dbg!(term1, ops);
    let mut term1_value = eval_term(term1);
    for (Op(op), term2) in ops {
        if !is_simple_arith_op(op.as_str()) {
            return None;
        }
        let term2_value = eval_term(term2);
        if let (Some(a), Some(b)) = (term1_value, term2_value) {
            term1_value = eval_binop(op, a, b);
        } else {
            return None;
        }
    }
    term1_value
}

fn is_simple_arith_op(op: &str) -> bool {
    ["+", "-", "*", "/", "|", "&", "~"].contains(&op)
}

fn optimize_expr(
    sym_table: Option<&SymbolTable>,
    term1: &Term,
    ops: &[(Op, Term)],
) -> Option<Vec<WriteInst>> {
    // dbg!(term1, ops);
    optimize_expr_int_inner(sym_table, term1, ops).map(signed_constant_to_constant_insts)
}

fn signed_constant_to_constant_insts(value: i16) -> Vec<WriteInst> {
    let mut insts = vec![write_push("constant", value.abs() as u16)];
    if value.is_negative() {
        insts.push(write_arith("neg"));
    }
    insts
}

fn eval_binop(op: &str, a: i16, b: i16) -> Option<i16> {
    // dbg!(op, a, b);
    match op {
        "+" => Some(a + b),
        "-" => Some(a - b),
        "*" => Some(a * b),
        "/" => Some(a / b),
        "|" => Some(a | b),
        "&" => Some(a & b),
        _ => None,
    }
}

fn eval_unop(op: &str, a: i16) -> Option<i16> {
    // dbg!(op, a, b);
    match op {
        "-" => eval_binop(op, 0, a),
        "~" => Some(if a == 0 { -1 } else { 0 }),
        _ => None,
    }
}

fn eval_term(term: &Term) -> Option<i16> {
    // dbg!(term);
    match term {
        &Term::IntegerConstant(value) => {
            // dbg!(value);
            Some(value.try_into().unwrap())
        }
        Term::UnaryOp(Op(op), term) => match eval_term(term) {
            Some(term_value) => eval_unop(op, term_value),
            None => None,
        },
        Term::ParenExpr(expr) => {
            let Expr(termp1, ops) = &**expr;
            optimize_expr_int_inner(None, termp1, ops)
        }
        Term::KeywordConstant(Keyword::Null) => Some(0),
        Term::KeywordConstant(Keyword::True) => Some(-1),
        Term::KeywordConstant(Keyword::False) => Some(0),
        _ => None,
    }
}
