use crate::{
    codegen::*,
    node::*,
    parser::ParseResult,
    symbol_table::{Entry, SubVarKind, SymbolTable},
    token::Keyword,
};
use std::{collections::HashSet, fmt::Write};

type CompilerError = Box<dyn std::error::Error>;
type Res<T = ()> = Result<T, CompilerError>;

struct CompilerState {
    class_name: String,
    label_id: usize,
    instructions: Vec<WriteInst>,
    methods: HashSet<String>,
    sym_table: SymbolTable,
}

impl CompilerState {
    fn new(class_name: String, sym_table: SymbolTable) -> Self {
        Self {
            class_name,
            label_id: 0,
            instructions: vec![],
            methods: Default::default(),
            sym_table,
        }
    }

    fn write_all(&mut self, out: &mut (dyn Write)) {
        for inst in self.instructions.iter() {
            self.write_inner(out, inst.code())
        }
    }

    fn write_inner<S: std::fmt::Display>(&self, out: &mut (dyn Write), s: S) {
        writeln!(out, "{}", s).expect("Error writing");
    }

    pub fn write_result(&mut self, res: WriteInst) {
        self.save_result(res);
    }

    pub fn get_label(&mut self) -> String {
        self.label_id += 1;
        Self::get_label_id(self.label_id)
    }

    fn get_label_id(label_id: usize) -> String {
        format!("__VM_LABEL_{}", label_id)
    }

    fn register_method(&mut self, sub_dec: &SubroutineDec) {
        if let SubroutineDec(GrammarSubroutineVariant::Method, _, ident, ..) = sub_dec {
            self.methods.insert(ident.into());
        };
    }

    fn has_method(&self, method: &str) -> bool {
        self.methods.contains(method)
    }

    fn save_result(&mut self, res: WriteInst) {
        self.instructions.push(res);
    }
}

#[derive(Debug, Default, Clone)]
struct CompilerContext {
    function_variant: Option<GrammarSubroutineVariant>,
    return_type: Option<GrammarSubroutineReturnType>,
}

impl CompilerContext {
    fn new() -> Self {
        Default::default()
    }
}

fn lookup_var(state: &mut CompilerState, context: &CompilerContext, name: String) -> Res<Entry> {
    let entry = state
        .sym_table
        .lookup(&name)
        .ok_or(format!("Unknown var: {}", &name))?;

    if let (Some(GrammarSubroutineVariant::Function), "this") =
        (&context.function_variant, entry.kind.as_str())
    {
        return Err(format!("Can't use field var in function: {}", name).into());
    };
    Ok(entry)
}

pub fn compile_program(parse_result: ParseResult) -> Res<String> {
    let mut out = String::new();
    let sym_table = SymbolTable::new();
    let mut state = CompilerState::new(Default::default(), sym_table);
    let context = CompilerContext::new();
    compile_class(&mut state, &context, parse_result.root).map_err(|e| {
        // dbg!(state.class_name, state.sym_table);
        e
    })?;
    state.write_all(&mut out);
    Ok(out)
}

fn compile_class(
    state: &mut CompilerState,
    context: &CompilerContext,
    Class(ident, var_decs, sub_decs): Class,
) -> Res {
    state.class_name = ident;
    for ClassVarDec(var_type, item_type, names) in var_decs {
        for name in names.iter() {
            state
                .sym_table
                .define_class_var(name, &var_type, &item_type);
        }
    }
    for sub_dec in sub_decs.iter() {
        state.register_method(sub_dec);
    }
    for sub_dec in sub_decs {
        compile_subroutine_dec(state, context, sub_dec)?;
    }
    Ok(())
}

fn compile_subroutine_dec(
    state: &mut CompilerState,
    context: &CompilerContext,
    SubroutineDec(variant, item_type, ident, params, sub): SubroutineDec,
) -> Res {
    state.sym_table.reset_subroutine_table();
    let n_locals: u16 = sub.0.iter().map(|var_dec| var_dec.1.len() as u16).sum();
    state.write_result(write_function(
        format!("{}.{}", state.class_name, ident),
        n_locals,
    ));
    let mut sub_context = context.clone();
    sub_context.function_variant = Some(variant.clone());
    sub_context.return_type = Some(item_type.clone());
    match variant {
        GrammarSubroutineVariant::Constructor => {
            let object_size = state.sym_table.count_instance_fields();
            state.write_result(write_push("constant", object_size));
            state.write_result(write_call("Memory.alloc", 1));
            state.write_result(write_pop("pointer", 0));
        }
        GrammarSubroutineVariant::Method => {
            state.write_result(write_push("argument", 0));
            state.write_result(write_pop("pointer", 0));

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

    compile_subroutine(state, &sub_context, sub, item_type)?;
    Ok(())
}

fn compile_subroutine(
    state: &mut CompilerState,
    context: &CompilerContext,
    Subroutine(var_decs, stmts): Subroutine,
    _typ: GrammarSubroutineReturnType,
) -> Res {
    for VarDec(type_, names) in var_decs {
        for name in names.iter() {
            state
                .sym_table
                .define_subroutine_var(name, SubVarKind::Var, &type_);
        }
    }
    compile_statements(state, context, stmts)?;
    Ok(())
}

fn compile_statements(
    state: &mut CompilerState,
    context: &CompilerContext,
    statements: Vec<Statement>,
) -> Res {
    for stmt in statements {
        compile_statement(state, context, stmt)?;
    }
    Ok(())
}

fn compile_statement(state: &mut CompilerState, context: &CompilerContext, stmt: Statement) -> Res {
    match stmt {
        Statement::LetStatement(s) => compile_statement_let(state, context, s)?,
        Statement::IfStatement(s) => compile_statement_if(state, context, s)?,
        Statement::WhileStatement(s) => compile_statement_while(state, context, s)?,
        Statement::DoStatement(s) => compile_statement_do(state, context, s)?,
        Statement::ReturnStatement(s) => compile_statement_return(state, context, s)?,
    };
    Ok(())
}

fn compile_statement_let(
    state: &mut CompilerState,
    context: &CompilerContext,
    stmt: LetStatement,
) -> Res {
    let var = state
        .sym_table
        .lookup(&stmt.name)
        .ok_or(format!("Unknown var: {}", &stmt.name))?;
    match stmt.index_expr {
        Some(expr) => {
            state.write_result(write_push(var.kind.as_str(), var.index));
            compile_expression(state, context, expr)?;
            state.write_result(write_arith("add"));
            compile_expression(state, context, stmt.value_expr)?;
            state.write_result(write_pop("temp", 0));
            state.write_result(write_pop("pointer", 1));
            state.write_result(write_push("temp", 0));
            state.write_result(write_pop("that", 0));
        }
        _ => {
            compile_expression(state, context, stmt.value_expr)?;
            state.write_result(write_pop(var.kind.as_str(), var.index));
        }
    };
    Ok(())
}

fn compile_statement_if(
    state: &mut CompilerState,
    context: &CompilerContext,
    stmt: IfStatement,
) -> Res {
    let end_label = state.get_label();
    let else_label = if stmt.else_statements.is_some() {
        state.get_label()
    } else {
        end_label.clone()
    };
    compile_expression(state, context, stmt.if_expr)?;
    state.write_result(write_not());
    state.write_result(write_if(&else_label));
    compile_statements(state, context, stmt.if_statements)?;
    state.write_result(write_goto(&end_label));
    if let Some(else_statements) = stmt.else_statements {
        state.write_result(write_label(&else_label));
        compile_statements(state, context, else_statements)?;
    }
    state.write_result(write_label(&end_label));
    Ok(())
}

fn compile_statement_while(
    state: &mut CompilerState,
    context: &CompilerContext,
    stmt: WhileStatement,
) -> Res {
    let start_label = state.get_label();
    let end_label = state.get_label();
    state.write_result(write_label(&start_label));
    compile_expression(state, context, stmt.cond_expr)?;
    state.write_result(write_not());
    state.write_result(write_if(&end_label));
    compile_statements(state, context, stmt.statements)?;
    state.write_result(write_goto(&start_label));
    state.write_result(write_label(&end_label));
    Ok(())
}

fn compile_statement_do(
    state: &mut CompilerState,
    context: &CompilerContext,
    stmt: DoStatement,
) -> Res {
    compile_call(state, context, stmt.call)?;
    // Pop return value, not used
    state.write_result(write_pop("temp", 0));
    Ok(())
}

fn compile_statement_return(
    state: &mut CompilerState,
    context: &CompilerContext,
    stmt: ReturnStatement,
) -> Res {
    // `return` statement validity check.
    match (&stmt.result, &context.return_type) {
        (Some(e), Some(GrammarSubroutineReturnType::Void)) => {
            return Err(format!("Expected void return, got: {:?}", e).into());
        }
        (None, Some(GrammarSubroutineReturnType::Type(t))) => {
            return Err(format!("Expected value return, got void. Expected type: {:?}", t).into());
        }
        _ => {}
    }
    match stmt.result {
        Some(expr) => {
            compile_expression(state, context, expr)?;
        }
        None => {
            state.write_result(write_push("constant", 0));
        }
    }
    state.write_result(write_return());
    Ok(())
}

fn compile_call(state: &mut CompilerState, context: &CompilerContext, call: SubroutineCall) -> Res {
    let (func_name, args) = match call {
        SubroutineCall::SimpleCall(method, args) => {
            if !state.has_method(&method) {
                return Err(format!("Can't call non-method as method: {}", method).into());
            };
            get_method_call(
                Term::KeywordConstant(Keyword::This),
                state.class_name.clone(),
                method,
                args,
            )
        }
        SubroutineCall::MethodCall(this_, method, args) => {
            let var = state.sym_table.lookup(&this_);
            match var {
                Some(entry) => get_method_call(Term::VarName(this_), entry.typ, method, args),
                None => (format!("{}.{}", this_, method), args),
            }
        }
    };
    let n_args = args.len();
    compile_expression_list(state, context, args)?;
    state.write_result(write_call(func_name, n_args));
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

fn compile_expression_list(
    state: &mut CompilerState,
    context: &CompilerContext,
    exprs: Vec<Expr>,
) -> Res {
    for expr in exprs {
        compile_expression(state, context, expr)?;
    }
    Ok(())
}

fn compile_expression(
    state: &mut CompilerState,
    context: &CompilerContext,
    Expr(term, extra_terms): Expr,
) -> Res {
    compile_term(state, context, term)?;
    for (op, extra_term) in extra_terms {
        compile_term(state, context, extra_term)?;
        compile_op(state, context, op)?;
    }
    Ok(())
}

fn compile_term(state: &mut CompilerState, context: &CompilerContext, term: Term) -> Res {
    match term {
        Term::VarName(name) => {
            let var = lookup_var(state, context, name)?;
            state.write_result(write_push(var.kind.as_str(), var.index));
        }
        Term::KeywordConstant(kw) => {
            match kw {
                Keyword::True => {
                    state.write_result(write_push("constant", 0));
                    state.write_result(write_not());
                }
                Keyword::False | Keyword::Null => {
                    state.write_result(write_push("constant", 0));
                }
                Keyword::This => {
                    state.write_result(write_push("pointer", 0));
                }
                _ => return Err(format!("Unexpected constant used as term: {:?}", kw).into()),
            };
        }
        Term::IntegerConstant(i) => {
            state.write_result(write_push("constant", i));
        }
        Term::StringConst(s) => {
            state.write_result(write_push("constant", s.len() as u16));
            state.write_result(write_call("String.new", 1));
            for c in s.chars() {
                state.write_result(write_push("constant", (c as u8).into()));
                state.write_result(write_call("String.appendChar", 2));
            }
        }
        Term::UnaryOp(op, term) => {
            compile_term(state, context, *term)?;
            compile_unary_op(state, context, op)?;
        }
        Term::ParenExpr(expr) => {
            compile_expression(state, context, *expr)?;
        }
        Term::IndexExpr(name, expr) => {
            let var = lookup_var(state, context, name)?;
            state.write_result(write_push(var.kind.as_str(), var.index));
            compile_expression(state, context, *expr)?;
            state.write_result(write_arith("add"));
            state.write_result(write_pop("pointer", 1));
            state.write_result(write_push("that", 0));
        }
        Term::SubroutineCall(call) => {
            compile_call(state, context, call)?;
        } // _t => todo!("Support term: {:?}", _t),
    }
    Ok(())
}

fn compile_op(state: &mut CompilerState, _context: &CompilerContext, Op(op): Op) -> Res {
    state.write_result(match op.as_str() {
        "+" => write_arith("add"),
        "-" => write_arith("sub"),
        "=" => write_arith("eq"),
        ">" => write_arith("gt"),
        "<" => write_arith("lt"),
        "&" => write_arith("and"),
        "|" => write_arith("or"),
        "*" => write_call("Math.multiply", 2),
        "/" => write_call("Math.divide", 2),
        other => unreachable!("Unsupported op: {:?}", other),
    });
    Ok(())
}

fn compile_unary_op(state: &mut CompilerState, _context: &CompilerContext, Op(op): Op) -> Res {
    state.write_result(match op.as_str() {
        "-" => write_arith("neg"),
        "~" => write_not(),
        other => unreachable!("Unsupported unary op: {:?}", other),
    });
    Ok(())
}
