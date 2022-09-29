use super::chain_reader::ChainReader;
use super::token::{Token, Type};
use super::utils;
use std::fmt::{Debug, Formatter, Result};

pub struct Lexer {
    string_reader: ChainReader<char>,
}

impl Lexer {
    pub fn new(code: String) -> Self {
        let chars = code.chars().collect::<Vec<char>>();
        Self {
            string_reader: ChainReader::new(chars),
        }
    }

    pub fn handle_special(&mut self, c: char) -> Option<Token> {
        let type_o = match c {
            '<' => Some(Type::BlockStart),
            '>' => Some(Type::BlockEnd),
            _ => None,
        };

        if type_o.is_some() {
            self.string_reader.advance();
            return Some(Token::new(c.to_string(), type_o.unwrap()));
        }

        None
    }

    pub fn handle_number(&mut self, c: char) -> Token {
        let mut raw = c.to_string();
        self.string_reader.advance();
        while let Some(current) = self.string_reader.get_current() {
            if !current.is_numeric() {
                break;
            }

            raw += &current.to_string();
            self.string_reader.advance();
        }

        Token::new(raw, Type::Number)
    }

    pub fn handle_identifer(&mut self, c: char) -> Token {
        let mut raw = c.to_string();
        self.string_reader.advance();
        while let Some(current) = self.string_reader.get_current() {
            if !utils::is_identifer(current) {
                break;
            }

            raw += &current.to_string();
            self.string_reader.advance();
        }

        Token::new(raw, Type::Identifier)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut raw = String::new();
        while let Some(current) = self.string_reader.get_current() {
            let mut token_o = None;
            let mut unvariable = false;
            if let Some(found_token) = self.handle_special(current) {
                token_o = Some(found_token);
            } else if current.is_numeric() {
                token_o = Some(self.handle_number(current));
            } else if current == '&' {
                // Identifiers starts with &
                token_o = Some(self.handle_identifer(current));
            } else {
                unvariable = true;
                raw += &current.to_string();
                self.string_reader.advance();
            }

            if !unvariable && raw != "" {
                let tmp_token = Token::new(raw.to_string(), Type::Unvariable);
                tokens.push(tmp_token);
                raw = String::new();
            }

            if let Some(token) = token_o {
                tokens.push(token);
            }
        }

        if raw != "" {
            let tmp_token = Token::new(raw.to_string(), Type::Unvariable);
            tokens.push(tmp_token);
        }

        tokens
    }
}

impl Debug for Lexer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Lexer")
            .field("string_reader", &self.string_reader)
            .finish()
    }
}
