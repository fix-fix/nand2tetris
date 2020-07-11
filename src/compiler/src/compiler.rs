use crate::{
    codegen::*,
    node::*,
    parser::ParseResult,
    symbol_table::{SubVarKind, SymbolTable},
    token::Keyword,
};
use std::fmt::Write;

type CompilerError = Box<dyn std::error::Error>;
type Res<T = ()> = Result<T, CompilerError>;

struct CompilerState<'a> {
    class_name: String,
    label_id: usize,
    sym_table: SymbolTable,
    out: &'a mut (dyn Write),
}

impl<'a> CompilerState<'a> {
    fn new(class_name: String, sym_table: SymbolTable, out: &'a mut (dyn Write)) -> Self {
        Self {
            class_name,
            label_id: 0,
            sym_table,
            out,
        }
    }

    pub fn write<S: std::fmt::Display>(&mut self, s: S) {
        writeln!(self.out, "{}", s).expect("Error writing");
    }

    pub fn get_label(&mut self) -> String {
        self.label_id += 1;
        format!("__VM_LABEL_{}", self.label_id)
    }
}

pub fn compile_program(parse_result: ParseResult) -> Res<String> {
    let mut out = String::new();
    let sym_table = SymbolTable::new();
    let mut state = CompilerState::new(Default::default(), sym_table, &mut out);
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
    let n_locals: u16 = sub.0.iter().map(|var_dec| var_dec.1.len() as u16).sum();
    state.write(write_function(
        format!("{}.{}", state.class_name, ident),
        n_locals,
    ));
    match variant {
        GrammarSubroutineVariant::Constructor => {
            let object_size = state.sym_table.count_instance_fields();
            state.write(write_push("constant", object_size));
            state.write(write_call("Memory.alloc", 1));
            state.write(write_pop("pointer", 0));
        }
        GrammarSubroutineVariant::Method => {
            state.write(write_push("argument", 0));
            state.write(write_pop("pointer", 0));

            // Offset arguments in methods by setting fake value, since we also pass 'this'
            state.sym_table.define_subroutine_var(
                &"this".to_string(),
                SubVarKind::Argument,
                &GrammarItemType::Class(state.class_name.clone()),
            );
        }
        _ => {}
    };
    for param in params.iter() {
        state
            .sym_table
            .define_subroutine_var(&param.ident, SubVarKind::Argument, &param.type_);
    }

    compile_subroutine(state, sub, item_type)?;
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
    Ok(())
}

fn compile_statements(state: &mut CompilerState, statements: Vec<Statement>) -> Res {
    for stmt in statements {
        compile_statement(state, stmt)?;
    }
    Ok(())
}

fn compile_statement(state: &mut CompilerState, stmt: Statement) -> Res {
    match stmt {
        Statement::LetStatement(s) => compile_statement_let(state, s)?,
        Statement::IfStatement(s) => compile_statement_if(state, s)?,
        Statement::WhileStatement(s) => compile_statement_while(state, s)?,
        Statement::DoStatement(s) => compile_statement_do(state, s)?,
        Statement::ReturnStatement(s) => compile_statement_return(state, s)?,
    };
    Ok(())
}

fn compile_statement_let(state: &mut CompilerState, stmt: LetStatement) -> Res {
    let var = state
        .sym_table
        .lookup(&stmt.name)
        .ok_or(format!("Unknown var: {}", &stmt.name))?;
    match stmt.index_expr {
        Some(expr) => {
            state.write(write_push(var.kind.as_str(), var.index));
            compile_expression(state, expr)?;
            state.write("add");
            compile_expression(state, stmt.value_expr)?;
            state.write(write_pop("temp", 0));
            state.write(write_pop("pointer", 1));
            state.write(write_push("temp", 0));
            state.write(write_pop("that", 0));
        }
        _ => {
            compile_expression(state, stmt.value_expr)?;
            state.write(write_pop(var.kind.as_str(), var.index));
        }
    };
    Ok(())
}

fn compile_statement_if(state: &mut CompilerState, stmt: IfStatement) -> Res {
    let end_label = state.get_label();
    let else_label = if stmt.else_statements.is_some() {
        state.get_label()
    } else {
        end_label.clone()
    };
    compile_expression(state, stmt.if_expr)?;
    state.write("not");
    state.write(write_if(&else_label));
    compile_statements(state, stmt.if_statements)?;
    state.write(write_goto(&end_label));
    if let Some(else_statements) = stmt.else_statements {
        state.write(write_label(&else_label));
        compile_statements(state, else_statements)?;
    }
    state.write(write_label(&end_label));
    Ok(())
}

fn compile_statement_while(state: &mut CompilerState, stmt: WhileStatement) -> Res {
    let start_label = state.get_label();
    let end_label = state.get_label();
    state.write(write_label(&start_label));
    compile_expression(state, stmt.cond_expr)?;
    state.write("not");
    state.write(write_if(&end_label));
    compile_statements(state, stmt.statements)?;
    state.write(write_goto(&start_label));
    state.write(write_label(&end_label));
    Ok(())
}

fn compile_statement_do(state: &mut CompilerState, stmt: DoStatement) -> Res {
    compile_call(state, stmt.call)?;
    // Pop return value, not used
    state.write(write_pop("temp", 0));
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
    let (func_name, args) = match call {
        SubroutineCall::SimpleCall(method, args) => get_method_call(
            Term::KeywordConstant(Keyword::This),
            state.class_name.clone(),
            method,
            args,
        ),
        SubroutineCall::MethodCall(this_, method, args) => {
            let var = state.sym_table.lookup(&this_);
            match var {
                Some(entry) => get_method_call(Term::VarName(this_), entry.typ, method, args),
                None => (format!("{}.{}", this_, method), args),
            }
        }
    };
    let n_args = args.len();
    compile_expression_list(state, args)?;
    state.write(write_call(func_name, n_args));
    Ok(())
}

fn get_method_call(
    var_term: Term,
    typ: String,
    method: String,
    args: Vec<Expr>,
) -> (String, Vec<Expr>) {
    let mut args_with_this = vec![Expr(var_term, vec![])];
    args_with_this.extend(args);
    (
        format!(
            "{}.{}",
            // Call actual class
            typ,
            method
        ),
        args_with_this,
    )
}

fn compile_expression_list(state: &mut CompilerState, exprs: Vec<Expr>) -> Res {
    for expr in exprs {
        compile_expression(state, expr)?;
    }
    Ok(())
}

fn compile_expression(state: &mut CompilerState, Expr(term, extra_terms): Expr) -> Res {
    compile_term(state, term)?;
    for (op, extra_term) in extra_terms {
        compile_term(state, extra_term)?;
        compile_op(state, op)?;
    }
    Ok(())
}

fn compile_term(state: &mut CompilerState, term: Term) -> Res {
    match term {
        Term::VarName(name) => {
            let var = state
                .sym_table
                .lookup(&name)
                .ok_or(format!("Unknown var: {}", &name))?;
            state.write(write_push(var.kind.as_str(), var.index));
        }
        Term::KeywordConstant(kw) => {
            match kw {
                Keyword::True => {
                    state.write(write_push("constant", 0));
                    state.write("not");
                }
                Keyword::False | Keyword::Null => {
                    state.write(write_push("constant", 0));
                }
                Keyword::This => {
                    state.write(write_push("pointer", 0));
                }
                _ => Err(format!("Unexpected constant used as term: {:?}", kw))?,
            };
        }
        Term::IntegerConstant(i) => {
            state.write(write_push("constant", i));
        }
        Term::StringConst(s) => {
            state.write(write_push("constant", s.len() as u16));
            state.write(write_call("String.new", 1));
            for c in s.chars() {
                state.write(write_push("constant", (c as u8).into()));
                state.write(write_call("String.appendChar", 2));
            }
        }
        Term::UnaryOp(op, term) => {
            compile_term(state, *term)?;
            compile_unary_op(state, op)?;
        }
        Term::ParenExpr(expr) => {
            compile_expression(state, *expr)?;
        }
        Term::IndexExpr(name, expr) => {
            let var = state
                .sym_table
                .lookup(&name)
                .ok_or(format!("Unknown var: {}", &name))?;
            state.write(write_push(var.kind.as_str(), var.index));
            compile_expression(state, *expr)?;
            state.write("add");
            state.write(write_pop("pointer", 1));
            state.write(write_push("that", 0));
        }
        Term::SubroutineCall(call) => {
            compile_call(state, call)?;
        } // _t => todo!("Support term: {:?}", _t),
    }
    Ok(())
}

fn compile_op(state: &mut CompilerState, Op(op): Op) -> Res {
    state.write(match op.as_str() {
        "+" => "add".to_string(),
        "-" => "sub".to_string(),
        "=" => "eq".to_string(),
        ">" => "gt".to_string(),
        "<" => "lt".to_string(),
        "&" => "and".to_string(),
        "|" => "or".to_string(),
        "*" => write_call("Math.multiply", 2),
        "/" => write_call("Math.divide", 2),
        other => unreachable!("Unsupported op: {:?}", other),
    });
    Ok(())
}

fn compile_unary_op(state: &mut CompilerState, Op(op): Op) -> Res {
    state.write(match op.as_str() {
        "-" => "neg".to_string(),
        "~" => "not".to_string(),
        other => unreachable!("Unsupported unary op: {:?}", other),
    });
    Ok(())
}
