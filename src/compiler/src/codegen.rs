pub fn write_pop(seg: &str, index: u16) -> String {
    format!("pop {} {}", seg, index)
}

pub fn write_push(seg: &str, index: u16) -> String {
    format!("push {} {}", seg, index)
}

pub fn write_function(name: String, n_locals: u16) -> String {
    format!("function {} {}", name, n_locals)
}

pub fn write_call<S: std::fmt::Display>(name: S, n_args: usize) -> String {
    format!("call {} {}", name, n_args)
}

pub fn write_return() -> String {
    "return".to_string()
}

pub fn write_label<S: std::fmt::Display>(label: S) -> String {
    format!("label {}", label)
}

pub fn write_if<S: std::fmt::Display>(label: S) -> String {
    format!("if-goto {}", label)
}

pub fn write_goto<S: std::fmt::Display>(label: S) -> String {
    format!("goto {}", label)
}
