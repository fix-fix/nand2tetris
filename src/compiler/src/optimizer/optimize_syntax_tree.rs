use crate::compiler::CompilerState;

use crate::codegen::{write_push, WriteInst};
use crate::node::{Expr, GrammarItemType, Op, Term};
use crate::symbol_table::{type_as_string, SymbolTable};

pub fn optimize_syntax_tree_expression(
    state: &mut CompilerState,
    expr: &Expr,
) -> Option<Vec<WriteInst>> {
    let Expr(term1, ops) = expr;
    if let Some((op, term2)) = ops.get(0) {
        return optimize_binop(state.sym_table(), op, term1, term2);
    }
    None
}

fn optimize_binop(
    sym_table: &SymbolTable,
    Op(op): &Op,
    term1: &Term,
    term2: &Term,
) -> Option<Vec<WriteInst>> {
    let op_s = &op.as_str();
    if ["+", "-", "*", "/"].contains(op_s) {
        match (term1, term2) {
            // dbg!(
            //     term1,
            //     term2,
            //     // sym_table.lookup(term1),
            //     // sym_table.lookup(term2),
            // );
            (Term::IntegerConstant(a), Term::IntegerConstant(b)) => {
                return eval_binop(op_s, *a, *b);
            }
            #[allow(unreachable_code)]
            (Term::VarName(va), Term::IntegerConstant(_b)) if false => {
                if let Some(entry) = sym_table.lookup(va) {
                    if entry.typ == type_as_string(&GrammarItemType::Int) {
                        dbg!(
                            term1,
                            term2,
                            sym_table.lookup(va),
                            // sym_table.lookup(term2)
                        );
                        return eval_binop(op_s, todo!(), *_b);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn eval_binop(op: &str, a: u16, b: u16) -> Option<Vec<WriteInst>> {
    match op {
        "+" => Some(vec![write_push("constant", a + b)]),
        "-" => Some(vec![write_push("constant", a - b)]),
        "*" => Some(vec![write_push("constant", a * b)]),
        "/" => Some(vec![write_push("constant", a / b)]),
        _ => None,
    }
}
