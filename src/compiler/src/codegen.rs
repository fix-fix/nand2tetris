pub fn write_pop(seg: &str, index: u16) -> String {
    format!("pop {} {}", seg, index)
}

pub fn write_push(seg: &str, index: u16) -> String {
    format!("push {} {}", seg, index)
}

pub fn write_function(name: String, n_locals: usize) -> String {
    format!("function {} {}", name, n_locals)
}

pub fn write_call<S: std::fmt::Display>(name: S, n_args: usize) -> String {
    format!("call {} {}", name, n_args)
}

pub fn write_return() -> String {
    "return".to_string()
}
