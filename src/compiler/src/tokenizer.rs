use crate::line_chars::LineChars;
use crate::token::*;
use crate::xml::*;

#[derive(Debug)]
pub struct Tokenizer<'a> {
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        let mut tokens: Vec<Token> = vec![];
        let mut chars = LineChars::new(self.source.char_indices());

        while let Some((_index, ch)) = chars.next() {
            let next_char = chars.clone().next();
            // dbg!((ch, _index, next_char, chars.line, chars.line_index));

            match ch {
                _ if ch.is_whitespace() => {}
                '\n' => {}
                '/' => {
                    match next_char {
                        None => {}
                        Some((_, '/')) => {
                            self.consume_until(&mut chars, "\n");
                        }
                        Some((_, '*')) => {
                            self.consume_until(&mut chars, "*/");
                        }
                        Some(..) => {
                            tokens.push(Token::Symbol(ch.to_string()));
                        }
                    };
                }
                // '/' is handled separately
                '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | ';' | '+' | '-' | '*' | '&'
                | '|' | '<' | '>' | '=' | '~' => tokens.push(Token::Symbol(ch.to_string())),
                '"' => {
                    // let buffer = String::new();
                }
                // _ if ch.is_numeric() => {
                //     // self.parse_numeric();
                // }
                // _ if ch.is_alphabetic() => {
                //     // self.parse_identifier();
                // }
                _ => {
                    return Err(format!(
                        "Can't tokenize at {1}:{2}: '{0}'",
                        ch, chars.line, chars.line_index
                    )
                    .into())
                }
            }
        }

        Ok(tokens)
    }

    fn consume_before(&self, chars: &mut LineChars, end: &str) {
        while !chars.as_str().starts_with(end) {
            chars.next();
        }
    }

    fn consume_n_chars(&self, chars: &mut LineChars, n: usize) {
        std::iter::repeat(()).take(n).zip(chars).last();
    }

    fn consume_until(&self, chars: &mut LineChars, end: &str) {
        self.consume_before(chars, end);
        self.consume_n_chars(chars, end.len());
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new(input);
    tokenizer.tokenize()
}

pub fn tokens_to_xml(tokens: Vec<Token>) -> String {
    xml_wrap_section(
        "tokens".into(),
        tokens
            .iter()
            .map(Token::as_xml_decl)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
