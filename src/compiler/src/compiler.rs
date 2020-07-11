use crate::{
    codegen::*,
    node::*,
    parser::ParseResult,
    symbol_table::{SubVarKind, SymbolTable},
};
use std::fmt::Write;

type CompilerError = Box<dyn std::error::Error>;
type Res<T = ()> = Result<T, CompilerError>;

struct CompilerState<'a> {
    class_name: String,
    sym_table: SymbolTable,
    out: &'a mut (dyn Write),
}

impl<'a> CompilerState<'a> {
    pub fn write<S: std::fmt::Display>(&mut self, s: S) {
        writeln!(self.out, "{}", s).expect("Error writing");
    }
}

pub fn compile_program(parse_result: ParseResult) -> Res<String> {
    let mut out = String::new();
    let sym_table = SymbolTable::new();
    let mut state = CompilerState {
        class_name: Default::default(),
        sym_table,
        out: &mut out,
    };
    dbg!(&parse_result.root);
    compile_class(&mut state, parse_result.root).map_err(|e| {
        dbg!(state.class_name, state.sym_table);
        e
    })?;
    Ok(out)
}

fn compile_class(state: &mut CompilerState, Class(ident, var_decs, sub_decs): Class) -> Res {
    state.class_name = ident;
    for ClassVarDec(var_type, item_type, names) in var_decs {
        for name in names.iter() {
            state
                .sym_table
                .define_class_var(name, &var_type, &item_type);
        }
    }
    for sub_dec in sub_decs {
        compile_subroutine_dec(state, sub_dec)?;
    }
    Ok(())
}

fn compile_subroutine_dec(
    state: &mut CompilerState,
    SubroutineDec(variant, item_type, ident, params, sub): SubroutineDec,
) -> Res {
    state.sym_table.reset_subroutine_table();
    let is_method = variant == GrammarSubroutineVariant::Method;
    state.write(write_function(
        format!("{}.{}", state.class_name, ident),
        params.len() + if is_method { 1 } else { 0 },
    ));
    for param in params.iter() {
        state
            .sym_table
            .define_subroutine_var(&param.ident, SubVarKind::Argument, &param.type_);
    }

    compile_subroutine(state, sub, item_type)?;
    // TODO: Handle methods
    Ok(())
}

fn compile_subroutine(
    state: &mut CompilerState,
    Subroutine(var_decs, stmts): Subroutine,
    typ: GrammarSubroutineReturnType,
) -> Res {
    // `return` statement validity check
    for return_stmt in stmts.iter().filter_map(|s| match s {
        Statement::ReturnStatement(rs) => Some(rs),
        _ => None,
    }) {
        match (&return_stmt.result, &typ) {
            (Some(e), GrammarSubroutineReturnType::Void) => {
                return Err(format!("Expected void return, got: {:?}", e))?;
            }
            (None, GrammarSubroutineReturnType::Type(t)) => {
                return Err(format!(
                    "Expected value return, got void. Expected type: {:?}",
                    t
                ))?;
            }
            _ => {}
        }
    }

    for VarDec(type_, names) in var_decs {
        for name in names.iter() {
            state
                .sym_table
                .define_subroutine_var(name, SubVarKind::Var, &type_);
        }
    }
    compile_statements(state, stmts)?;
    // todo!();
    Ok(())
}

fn compile_statements(state: &mut CompilerState, statements: Vec<Statement>) -> Res {
    for stmt in statements {
        compile_statement(state, stmt)?;
    }
    // todo!();
    Ok(())
}

fn compile_statement(state: &mut CompilerState, stmt: Statement) -> Res {
    match stmt {
        Statement::LetStatement(s) => compile_statement_let(state, s)?,
        Statement::IfStatement(_) => todo!(),
        Statement::WhileStatement(_) => todo!(),
        Statement::DoStatement(s) => compile_statement_do(state, s)?,
        Statement::ReturnStatement(s) => compile_statement_return(state, s)?,
    };
    Ok(())
}

fn compile_statement_let(state: &mut CompilerState, stmt: LetStatement) -> Res {
    // TODO: Resolve symbol
    todo!();
    Ok(())
}

fn compile_statement_do(state: &mut CompilerState, stmt: DoStatement) -> Res {
    // FIXME: Handle void calls
    compile_call(state, stmt.call)?;
    // Pop return value, not used
    state.write(write_pop("temp", 0));
    // todo!();
    Ok(())
}

fn compile_statement_return(state: &mut CompilerState, stmt: ReturnStatement) -> Res {
    match stmt.result {
        Some(expr) => {
            compile_expression(state, expr)?;
        }
        None => {
            state.write(write_push("constant", 0));
        }
    }
    state.write(write_return());
    Ok(())
}

fn compile_call(state: &mut CompilerState, call: SubroutineCall) -> Res {
    // TODO Handle functions/methods/constructors
    let (func_name, args) = match call {
        SubroutineCall::SimpleCall(_, _args) => todo!(),
        SubroutineCall::MethodCall(this_, method, args) => {
            let var = state.sym_table.lookup(&this_);
            match var {
                // TODO: Prepend this to args, if it's an object
                Some(_) => todo!(),
                None => (format!("{}.{}", this_, method), args),
            }
        }
    };
    let n_args = args.len();
    compile_expression_list(state, args)?;
    // TODO: Handle n_args when passing this
    state.write(write_call(func_name, n_args));
    Ok(())
}

fn compile_expression_list(state: &mut CompilerState, exprs: Vec<Expr>) -> Res {
    for expr in exprs {
        compile_expression(state, expr)?;
    }
    // todo!();
    Ok(())
}

fn compile_expression(state: &mut CompilerState, Expr(term, extra_terms): Expr) -> Res {
    compile_term(state, term)?;
    for (op, extra_term) in extra_terms {
        compile_term(state, extra_term)?;
        compile_op(state, op)?;
    }
    // todo!();
    Ok(())
}

fn compile_term(state: &mut CompilerState, term: Term) -> Res {
    match term {
        // Term::VarName(_) => {}
        // Term::KeywordConstant(_) => {}
        Term::IntegerConstant(i) => {
            state.write(write_push("constant", i));
        }
        // Term::StringConst(_) => {}
        // Term::UnaryOp(_, _) => {}
        Term::ParenExpr(expr) => {
            compile_expression(state, *expr)?;
        }
        // Term::IndexExpr(_, _) => {}
        // Term::SubroutineCall(_) => {}
        _t => todo!("Support term: {:?}", _t),
    }
    Ok(())
}

fn compile_op(state: &mut CompilerState, Op(op): Op) -> Res {
    match op.as_str() {
        "+" => {
            state.write("add");
        }
        "*" => {
            state.write(write_call("Math.multiply", 2));
        }
        other => todo!("Support op: {:?}", other),
    }
    Ok(())
}
