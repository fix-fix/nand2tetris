use crate::compiler::{CompilerContext, CompilerState, LhsContext, LhsContextInner};

use crate::codegen::{write_arith, write_push, WriteInst};
use crate::node::{Expr, GrammarItemType, Op, Term};
use crate::symbol_table::{type_as_string, SymbolTable};
use crate::token::Keyword;

pub fn optimize_syntax_tree_expression(
    Expr(term1, ops): &Expr,
    state: &mut CompilerState,
    context: &CompilerContext,
) -> Option<Vec<WriteInst>> {
    let lhs_context = context.lhs_context();
    optimize_expr(term1, ops, state.sym_table(), &lhs_context)
}

fn optimize_expr_int_inner(
    term1: &Term,
    ops: &[(Op, Term)],
    sym_table: &mut SymbolTable,
    lhs_context: &LhsContext,
) -> Option<i16> {
    // dbg!(term1, ops);
    let mut term1_value = eval_term(term1, sym_table, lhs_context);
    for (Op(op), term2) in ops {
        if !is_simple_arith_op(op.as_str()) {
            return None;
        }
        let term2_value = eval_term(term2, sym_table, lhs_context);
        if let (Some(a), Some(b)) = (term1_value, term2_value) {
            term1_value = eval_binop(op, a, b);
        } else {
            return None;
        }
    }

    if let (Some(term_value), Some(LhsContextInner::ClassStatic(class_static))) =
        (term1_value, lhs_context)
    {
        sym_table.add_constant_value_for_static(class_static.name(), term_value);
    }

    term1_value
}

fn is_simple_arith_op(op: &str) -> bool {
    ["+", "-", "*", "/", "|", "&", "~"].contains(&op)
}

fn optimize_expr(
    term1: &Term,
    ops: &[(Op, Term)],
    sym_table: &mut SymbolTable,
    lhs_context: &LhsContext,
) -> Option<Vec<WriteInst>> {
    // dbg!(term1, ops);
    optimize_expr_int_inner(term1, ops, sym_table, lhs_context)
        .map(signed_constant_to_constant_insts)
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

fn eval_term(term: &Term, sym_table: &mut SymbolTable, lhs_context: &LhsContext) -> Option<i16> {
    // dbg!(term);
    match term {
        Term::IntegerConstant(value) => {
            // dbg!(value);
            Some((*value).try_into().unwrap())
        }
        Term::UnaryOp(Op(op), term) => match eval_term(term, sym_table, lhs_context) {
            Some(term_value) => eval_unop(op, term_value),
            None => None,
        },
        Term::ParenExpr(expr) => {
            let Expr(termp1, ops) = &**expr;
            optimize_expr_int_inner(termp1, ops, sym_table, lhs_context)
        }
        Term::KeywordConstant(Keyword::Null) => Some(0),
        Term::KeywordConstant(Keyword::True) => Some(-1),
        Term::KeywordConstant(Keyword::False) => Some(0),
        #[allow(clippy::let_and_return)]
        Term::VarName(ident) => {
            let entry = sym_table.lookup(ident)?;
            if entry.typ != type_as_string(&GrammarItemType::Int) {
                return None;
            }
            let constant_value = entry.constant_value();
            // dbg!(constant_value, ident, entry);
            constant_value
        }
        _ => None,
    }
}
