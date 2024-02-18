use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use peekmore::PeekMore;

#[derive(Debug, Clone, thiserror::Error)]
pub enum TokenError {
    #[error("{0}")]
    LexingError(String),
}

#[derive(Debug, Clone)]
pub enum Token {
    // Element
    /// <
    ElementTagStartOpen,
    /// </
    ElementTagEndOpen,
    /// >
    ElementTagCloseMany,
    /// />
    ElementTagCloseSingle,

    // Comments
    /// <!-- COMMENT -->
    Comment(String),

    // Processing instruction
    /// <? PROCESSING_INSTRUCTION ?>
    ProcessingInstruction(String),

    // General
    /// <IDENTIFIER> or <IDENTIFIER IDENTIFIER="VALUE">
    Identifier(String),
    /// <IDENTIFIER IDENTIFIER="VALUE">
    Value(String),
    /// A "="
    Equals,
    /// A Linebreak "\n" or "\n\r" on windows
    Linebreak,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::ElementTagStartOpen => write!(f, "<"),
            Token::ElementTagEndOpen => write!(f, "</"),
            Token::ElementTagCloseMany => write!(f, ">"),
            Token::ElementTagCloseSingle => write!(f, "/>"),
            Token::Comment(s) => write!(f, "Comment: {}", s),
            Token::ProcessingInstruction(s) => write!(f, "PI: {}", s),
            Token::Identifier(s) => write!(f, "Identifier: {}", s),
            Token::Value(s) => write!(f, "Value: {}", s),
            Token::Equals => write!(f, "="),
            Token::Linebreak => write!(f, ""),
        }
    }
}

pub struct Tokenizer<T: std::io::Read> {
    buffer: BufReader<T>,
    chars: Vec<char>,
    index: usize,
}

impl<T: std::io::Read> Tokenizer<T> {
    pub fn new(buffer: BufReader<T>) -> Self {
        Self {
            buffer,
            chars: Vec::<char>::new(),
            index: 0usize,
        }
    }
}

impl<T: std::io::Read> Iterator for Tokenizer<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.chars.len() {
            let mut string_buf = String::new();
            if let Ok(0) = self.buffer.read_line(&mut string_buf) {
                return None;
            }
            self.chars = string_buf.chars().collect();
            self.index = 0usize;
        }
        self.index += 1;
        Some(self.chars[self.index - 1])
    }
}

impl<T: std::io::Read> Tokenizer<T> {
    pub fn tokenize(self) -> Result<Vec<Token>, TokenError> {
        let mut tokens = Vec::<Token>::new();
        let mut iter = self.peekmore();

        let mut buf = String::new();
        let mut skip = 0u64;
        let mut lexing_value = false;
        let mut lexing_comment = false;
        let mut lexing_pi = false;

        while let Some(char) = iter.next() {
            if skip != 0 {
                skip -= 1;
                continue;
            }

            if lexing_value {
                if char == '"' || char == '\'' {
                    lexing_value = false;
                    tokens.push(Token::Value(buf));
                    buf = String::new();
                    continue;
                }
                buf.push(char);
                continue;
            }

            if lexing_comment {
                let dash1 = *iter.peek().unwrap_or(&'i');
                iter.advance_cursor();
                let dash2 = *iter.peek().unwrap_or(&'i');
                iter.advance_cursor();
                let close = *iter.peek().unwrap_or(&'i');

                if char == ' ' && dash1 == '-' && dash2 == '-' && close == '>' {
                    lexing_comment = false;
                    skip = 3;
                    tokens.push(Token::Comment(buf));
                    buf = String::new();
                    continue;
                }
                iter.reset_cursor();
                buf.push(char);
                continue;
            }

            if lexing_pi {
                let question_mark = *iter.peek().unwrap_or(&'i');
                iter.advance_cursor();
                let close = *iter.peek().unwrap_or(&'i');
                if char == ' ' && question_mark == '?' && close == '>' {
                    lexing_pi = false;
                    skip = 2;
                    tokens.push(Token::ProcessingInstruction(buf));
                    buf = String::new();
                    continue;
                }
                iter.reset_cursor();
                buf.push(char);
                continue;
            }

            // move actual character to the top because the compiler can use them as an index into an array with addresses to jump to
            match char {
                // Possibly ElementTagStartOpen, ElementTagEndOpen, Comment or PI
                '<' => {
                    match iter.peek().unwrap() {
                        // A slash follows. It must be ElementTagEndOpen
                        '/' => {
                            tokens.push(Token::ElementTagEndOpen);
                            skip = 1;
                        }
                        // An exclamation mark follows. It might be a comment
                        '!' => {
                            iter.advance_cursor();
                            let dash1 = *iter.peek().unwrap_or(&'i');
                            iter.advance_cursor();
                            let dash2 = *iter.peek().unwrap_or(&'i');
                            iter.advance_cursor();
                            let whitespace = *iter.peek().unwrap_or(&'i');

                            if dash1 != '-' || dash2 != '-' || whitespace != ' ' {
                                return Err(TokenError::LexingError(
                                    "Expected comment after exclamation mark.".to_string(),
                                ));
                            }

                            // It's a comment
                            skip = 4;
                            lexing_comment = true;
                            iter.reset_cursor();
                        }
                        // It's a PI
                        '?' => {
                            skip = 2;
                            lexing_pi = true;
                            iter.reset_cursor();
                        }
                        // Has an identifier. It must be ElementTagStartOpen
                        c if c.is_alphanumeric() => tokens.push(Token::ElementTagStartOpen),
                        _ => {
                            return Err(TokenError::LexingError(format!(
                                "Unknown symbol '{}' after '<'.",
                                char
                            )))
                        }
                    }
                }
                // Possibly ElementTagCloseSingle or ElementTagEndOpen
                '/' => match iter.peek().unwrap() {
                    '>' => {
                        if !buf.is_empty() {
                            tokens.push(Token::Identifier(buf));
                            buf = String::new();
                        }
                        tokens.push(Token::ElementTagCloseSingle);
                        skip = 1;
                    }
                    _ => {
                        return Err(TokenError::LexingError(format!(
                            "Unknown symbol '{0}' after '/'",
                            char
                        )))
                    }
                },
                // ElementTagCloseMany
                '>' => {
                    if !buf.is_empty() {
                        tokens.push(Token::Identifier(buf));
                        buf = String::new();
                    }
                    tokens.push(Token::ElementTagCloseMany);
                }
                // Linebreak
                '\n' => {
                    if iter.peek() == Some(&'\r') {
                        skip = 1;
                    }
                    if !buf.is_empty() {
                        tokens.push(Token::Identifier(buf));
                        buf = String::new();
                    }
                    tokens.push(Token::Linebreak);
                }
                // Equals
                '=' => {
                    if !buf.is_empty() {
                        tokens.push(Token::Identifier(buf));
                        buf = String::new();
                    }
                    tokens.push(Token::Equals)
                }
                // Start Value lexing
                '"' | '\'' => lexing_value = true,
                // Identifier
                c if c.is_alphanumeric() => {
                    buf.push(c);
                }
                // Whitespace
                c if c.is_whitespace() => {
                    if !buf.is_empty() {
                        tokens.push(Token::Identifier(buf));
                        buf = String::new();
                    }
                }
                _ => {
                    return Err(TokenError::LexingError(format!(
                        "Unknown symbol '{}'",
                        char
                    )))
                }
            }
        }

        Ok(tokens)
    }
}
