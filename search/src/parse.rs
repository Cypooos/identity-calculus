use std::str::FromStr;

use thiserror::Error;

use crate::tree::Tree;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TreeParseError {
    #[error("expected '(' at byte {at}")]
    ExpectedOpenParen { at: usize },
    #[error("expected ')' at byte {at}")]
    ExpectedCloseParen { at: usize },
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("trailing characters at byte {at}")]
    TrailingCharacters { at: usize },
}

/// Parse un arbre depuis la syntaxe canonique `()` / `( ... )` (en ignorant les espaces).
pub fn parse_tree(s: &str) -> Result<Tree, TreeParseError> {
    let mut parser = Parser::new(s);
    let tree = parser.parse_tree()?;
    parser.skip_ws();
    if let Some(at) = parser.pos_if_remaining() {
        return Err(TreeParseError::TrailingCharacters { at });
    }
    Ok(tree)
}

impl FromStr for Tree {
    type Err = TreeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_tree(s)
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            pos: 0,
        }
    }

    fn pos_if_remaining(&self) -> Option<usize> {
        (self.pos < self.bytes.len()).then_some(self.pos)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        self.skip_ws();
        let b = self.bytes.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }

    fn parse_tree(&mut self) -> Result<Tree, TreeParseError> {
        let at = self.pos;
        match self.bump() {
            Some(b'(') => {}
            Some(_) => return Err(TreeParseError::ExpectedOpenParen { at }),
            None => return Err(TreeParseError::UnexpectedEof),
        }

        let mut children = Vec::new();
        loop {
            match self.peek() {
                Some(b')') => {
                    let at = self.pos;
                    match self.bump() {
                        Some(b')') => break,
                        _ => return Err(TreeParseError::ExpectedCloseParen { at }),
                    }
                }
                Some(b'(') => {
                    let child = self.parse_tree()?;
                    children.push(child);
                }
                Some(_) => return Err(TreeParseError::ExpectedCloseParen { at: self.pos }),
                None => return Err(TreeParseError::UnexpectedEof),
            }
        }

        Ok(Tree { children })
    }
}

