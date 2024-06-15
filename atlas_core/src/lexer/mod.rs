use std::{collections::HashMap, iter::Peekable, str::Chars};

use internment::Intern;
use token::{Token, TokenKind};

use crate::{
    map, parser,
    utils::span::{BytePos, Span},
};

pub mod token;

pub struct Lexer<'l> {
    path: &'static str,
    current_pos: BytePos,
    it: Peekable<Chars<'l>>,
    keywords: HashMap<Intern<String>, TokenKind>,
}

impl<'l> Lexer<'l> {
    pub fn tokenize(path: &'static str, contents: &'l str) -> Result<Vec<Token>, String> {
        let mut lexer = Self {
            path,
            current_pos: BytePos::default(),
            it: contents.chars().peekable(),
            keywords: HashMap::new(),
        };
        lexer.populate_keyword();
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            let start_pos = lexer.current_pos;
            let ch = match lexer.it.next() {
                Some(c) => c,
                None => break,
            };

            match lexer.lex(ch) {
                Ok(kind) => {
                    tokens.push(Token::new(
                        Span {
                            start: start_pos,
                            end: lexer.current_pos,
                            path: lexer.path,
                        },
                        kind,
                    ));
                    if kind == TokenKind::EoI {
                        break;
                    }
                }
                Err(msg) => {
                    println!("{}:{}: {}", lexer.path, lexer.current_pos, msg);
                    return Err(msg);
                }
            }
        }

        Ok(tokens)
    }

    fn next(&mut self) -> Option<char> {
        let nxt = self.it.next();
        if let Some(nxt) = nxt {
            self.current_pos = self.current_pos.shift(nxt);
        }
        nxt
    }

    #[inline(always)]
    fn peek(&mut self) -> Option<&char> {
        self.it.peek()
    }

    fn either(&mut self, to_match: char, matched: TokenKind, unmatched: TokenKind) -> TokenKind {
        if self.consume_if(|c| c == to_match) {
            matched
        } else {
            unmatched
        }
    }

    fn consume_if<F>(&mut self, f: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        if let Some(&ch) = self.it.peek() {
            if f(ch) {
                self.next().unwrap();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn consume_if_next<F>(&mut self, f: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        let mut it = self.it.clone();
        match it.next() {
            None => return false,
            _ => (),
        }

        if let Some(&ch) = it.peek() {
            if f(ch) {
                self.next().unwrap();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn consume_while<F>(&mut self, f: F) -> Vec<char>
    where
        F: Fn(char) -> bool,
    {
        let mut chars: Vec<char> = Vec::new();
        while let Some(&ch) = self.peek() {
            if f(ch) {
                self.next().unwrap();
                chars.push(ch);
            } else {
                break;
            }
        }
        chars
    }

    fn lex(&mut self, ch: char) -> Result<TokenKind, String> {
        use TokenKind::*;
        match ch {
            '\n' | '\t' | ' ' | '\r' => {
                if !self.peek().is_none() {
                    let ch = self.next().unwrap();
                    self.lex(ch)
                } else {
                    Err("Unexpected end of input".to_string())
                }
            }
            '(' => Ok(LParen),
            ')' => Ok(RParen),
            '{' => Ok(LBrace),
            '}' => Ok(RBrace),
            '[' => Ok(LBracket),
            ']' => Ok(RBracket),
            '+' => Ok(Plus),
            '_' => Ok(Underscore),
            '-' => Ok(self.either('>', RArrow, Minus)),
            '*' => Ok(Star),
            //TODO: Add support for multiline comments
            '/' => {
                if self.consume_if(|c| c == '/') {
                    self.consume_while(|c| c != '\n');
                    if !self.peek().is_none() {
                        let ch = self.next().unwrap();
                        self.lex(ch)
                    } else {
                        Err("Unexpected end of input".to_string())
                    }
                } else {
                    Ok(Slash)
                }
            }
            '\\' => {
                //Add support for escaping characters
                Ok(Backslash)
            }
            '%' => Ok(Percent),
            '^' => Ok(Caret),
            '<' => {
                if self.consume_if(|c| c == '=') {
                    Ok(LtEq)
                } else {
                    Ok(self.either('-', LArrow, LAngle))
                }
            }
            '>' => Ok(self.either('=', GtEq, RAngle)),
            '=' => {
                if self.consume_if(|ch| ch == '>') {
                    Ok(FatArrow)
                } else {
                    Ok(self.either('=', DoubleEq, Eq))
                }
            }
            '&' => Ok(Ampersand),
            '|' => Ok(Pipe),
            '!' => Ok(self.either('=', NEq, Bang)),
            //Logical
            ':' => Ok(self.either(':', DoubleColon, Colon)),
            ';' => Ok(SemiColon),
            ',' => Ok(Comma),
            '.' => Ok(self.either('.', DoubleDot, Dot)),
            '@' => Ok(At),
            '#' => Ok(HashTag),
            '~' => Ok(Tilde),
            '?' => Ok(Question),
            '$' => Ok(Dollar),
            //Identifiers
            ch if ch.is_alphabetic() || ch == '_' => Ok(self.identifier(ch).unwrap()),
            x if x.is_numeric() => Ok(self.number(x).unwrap()),
            '"' => {
                let mut string = String::new();
                string.push_str(
                    self.consume_while(|ch| ch != '"')
                        .iter()
                        .collect::<String>()
                        .as_ref(),
                );
                match self.next() {
                    Some('"') => (),
                    _ => {
                        println!("{}:{}", self.current_pos, self.path);
                        return Err("Unterminated string literal".to_string())
                    },
                }
                Ok(TokenKind::Literal(
                    self::token::TokenLiteral::StringLiteral(Intern::new(string)),
                ))
            }
            c => Err("Unexpected character".to_string()),
        }
    }

    fn identifier(&mut self, c: char) -> Option<TokenKind> {
        let mut ident = String::new();
        ident.push(c);

        while let Some(&ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.next().unwrap());
            } else {
                break;
            }
        }
        let id = Intern::new(ident.to_owned());

        if let Some(k) = self.keywords.get(&id) {
            Some(k.clone())
        } else {
            Some(TokenKind::Literal(self::token::TokenLiteral::Identifier(
                id,
            )))
        }
    }

    fn number(&mut self, c: char) -> Option<TokenKind> {
        let mut number = String::new();
        number.push(c);

        let num: String = self.consume_while(|a| a.is_numeric()).into_iter().collect();
        number.push_str(&num);

        if self.peek() == Some(&'.') && self.consume_if_next(|c| c.is_numeric()) {
            number.push('.');

            let num: String = self.consume_while(|a| a.is_numeric()).into_iter().collect();
            number.push_str(&num);
        }
        Some(TokenKind::Literal(self::token::TokenLiteral::Number(
            number.parse::<f64>().unwrap(),
        )))
    }

    fn populate_keyword(&mut self) {
        self.keywords = map! {
            Intern::new(String::from("match")) => TokenKind::Keyword(Intern::new(String::from("match"))),
            Intern::new(String::from("as")) => TokenKind::Keyword(Intern::new(String::from("as"))),
            Intern::new(String::from("enum")) => TokenKind::Keyword(Intern::new(String::from("enum"))),
            Intern::new(String::from("do")) => TokenKind::Keyword(Intern::new(String::from("do"))),
            Intern::new(String::from("with")) => TokenKind::Keyword(Intern::new(String::from("with"))),
            Intern::new(String::from("or")) => TokenKind::Keyword(Intern::new(String::from("or"))),
            Intern::new(String::from("And")) => TokenKind::Keyword(Intern::new(String::from("and"))),
            Intern::new(String::from("struct")) => TokenKind::Keyword(Intern::new(String::from("struct"))),
            Intern::new(String::from("let")) => TokenKind::Keyword(Intern::new(String::from("let"))),
            Intern::new(String::from("fn")) => TokenKind::Keyword(Intern::new(String::from("fn"))),
            Intern::new(String::from("in")) => TokenKind::Keyword(Intern::new(String::from("in")))

        }
    }
}
