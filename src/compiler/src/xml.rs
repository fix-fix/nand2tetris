pub fn xml_wrap_section(name: &str, s: &str) -> String {
    format!("<{name}>\n{s}\n</{name}>", name = name, s = s)
}

pub fn xml_wrap_section_lines(name: &str, s: &str) -> Vec<String> {
    vec![
        format!("<{name}>", name = name),
        s.to_string(),
        format!("</{name}>", name = name),
    ]
}

pub fn xml_wrap_declaration(name: &str, s: &str) -> String {
    format!(
        "<{name}> {s} </{name}>",
        name = name,
        s = replace_xml_entitires(s)
    )
}

fn replace_xml_entitires(s: &str) -> String {
    let result = s.to_string();
    result
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}
