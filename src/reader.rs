use crate::token::*;
use crate::context::*;
use crate::utils::dart_parseerror;

pub struct Reader {
    pos: usize,
    tokens: Vec<Token>
}

impl Reader {

    pub fn new(tokens: Vec<Token>) -> Reader {
        Reader {
            pos: 0,
            tokens
        }
    }

    pub fn expect(&self, sym: &str, ctx: &Ctx) -> Result<(), String> {
        if let Some(t) = self.tokens.get(self.pos) {
            if format!("{}", t) != sym {
                dart_parseerror(
                    format!("Expected: '{}'. Got: '{}'.", sym, t),
                    ctx,
                    self.tokens(),
                    self.pos()
                );
                Err(format!("Expected: '{}'. Got: '{}'.", sym, t))
            } else {
                Ok(())
            }
        } else {
            dart_parseerror(
                format!("Index out of bounds while expecting symbol: '{}'", sym),
                ctx,
                self.tokens(),
                self.pos()
            );
            Err(format!("Index out of bounds while expecting symbol: '{}'", sym))
        }
    }

    pub fn skip(&mut self, sym: &str, ctx: &Ctx) -> Result<(), String> {
        self.expect(sym, ctx)?;
        self.next();
        Ok(())
    }

    pub fn sym(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    pub fn next(&mut self) -> Option<Token> {
        self.pos += 1;
        self.tokens.get(self.pos).cloned()
    }

    pub fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos + 1).cloned()
    }

    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn more(&self) -> bool {
        self.len() > self.pos + 1
    }
}