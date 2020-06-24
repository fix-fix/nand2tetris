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

        while let Some(ch) = chars.peek() {
            // dbg!((ch, chars.line, chars.line_index));

            match ch {
                _ if ch.is_whitespace() => {
                    chars.next();
                }
                '\n' => {
                    chars.next();
                }
                '/' => {
                    chars.next();
                    let next_char = chars.peek();
                    match next_char {
                        None => {}
                        Some('/') => {
                            chars.next();
                            Self::consume_until(&mut chars, "\n");
                        }
                        Some('*') => {
                            chars.next();
                            Self::consume_until(&mut chars, "*/");
                        }
                        Some(..) => {
                            tokens.push(Token::Symbol(ch.to_string()));
                        }
                    };
                }
                // '/' is handled separately
                '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | ';' | '+' | '-' | '*' | '&'
                | '|' | '<' | '>' | '=' | '~' => {
                    tokens.push(Token::Symbol(ch.to_string()));
                    chars.next();
                }
                '"' => {
                    chars.next();
                    tokens.push(Self::parse_string(&mut chars));
                }
                _ if ch.is_numeric() => {
                    tokens.push(Self::parse_numeric(&mut chars)?);
                }
                _ if Self::is_identifier_start_char(ch) => {
                    tokens.push(Self::parse_identifier_or_keyword(&mut chars)?);
                }
                _ => {
                    return self.tokenization_error(&mut chars, ch);
                }
            }
        }

        Ok(tokens)
    }

    fn parse_identifier_or_keyword(
        chars: &mut LineChars,
    ) -> Result<Token, Box<dyn std::error::Error>> {
        let s = Self::consume_while(chars, Self::is_identifier_char);
        Ok(
            if let Some(token_keyword) = keyword_from_string(s.as_str()) {
                Token::Keyword(token_keyword)
            } else {
                Token::Identifier(s)
            },
        )
    }

    fn parse_string(chars: &mut LineChars) -> Token {
        let s = chars
            .map(|x| x.1)
            .take_while(|ch| *ch != '"')
            .collect::<String>();
        Token::StringConst(s)
    }

    fn parse_numeric(chars: &mut LineChars) -> Result<Token, Box<dyn std::error::Error>> {
        let num = Self::consume_while(chars, |ch| ch.is_numeric());
        str::parse::<u16>(num.as_str())
            .or_else(|_| Err(format!("Can't parse num: {}", num).into()))
            .map(Token::IntegerConst)
    }

    fn is_identifier_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn is_identifier_start_char(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn consume_while<P>(chars: &mut LineChars, predicate: P) -> String
    where
        P: Fn(char) -> bool,
    {
        std::iter::from_fn(|| {
            let ch = chars.peek()?;
            if predicate(ch) {
                chars.next();
                Some(ch)
            } else {
                None
            }
        })
        .collect()
    }

    fn consume_before(chars: &mut LineChars, end: &str) {
        while !chars.as_str().starts_with(end) {
            chars.next();
        }
    }

    fn consume_n_chars(chars: &mut LineChars, n: usize) {
        chars.nth(n - 1);
    }

    fn consume_until(chars: &mut LineChars, end: &str) {
        Self::consume_before(chars, end);
        Self::consume_n_chars(chars, end.len());
    }

    fn tokenization_error(
        &self,
        chars: &mut LineChars,
        ch: char,
    ) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        chars.next();
        // dbg!(&tokens.iter().rev().take(30).rev().collect::<Vec<_>>());
        Err(format!(
            "Can't tokenize at {1}:{2}: '{0}'\n\
            At line:\n\
            {line}\n",
            ch,
            chars.line,
            chars.line_index,
            line = self.source.lines().nth(chars.line - 1).unwrap_or_default()
        )
        .into())
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new(input);
    tokenizer.tokenize()
}

pub fn tokens_to_xml(tokens: Vec<Token>) -> String {
    xml_wrap_section(
        "tokens",
        tokens
            .iter()
            .map(Token::as_xml_decl)
            .collect::<Vec<_>>()
            .join("\n")
            .as_str(),
    )
}
