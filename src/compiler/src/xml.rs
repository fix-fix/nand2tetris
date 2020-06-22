pub fn xml_wrap_section(name: String, s: String) -> String {
    format!("<{name}>\n{s}\n</{name}>", name = name, s = s)
}

pub fn xml_wrap_declaration(name: String, s: String) -> String {
    format!("<{name}> {s} </{name}>", name = name, s = s)
}
