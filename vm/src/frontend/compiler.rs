use num_derive::FromPrimitive;
use num_traits::FromPrimitive as _;

use crate::common::*;
use crate::debug::dissassemble_chunk;
use crate::frontend::scanner::*;
use crate::chunk::*;
use crate::backend::vm::*;
use crate::value::*;

struct Parser<'src> {
    current:    Token<'src>, 
    previous:   Token<'src>, 
    had_error:  bool,
    panic_mode: bool,
}

pub struct Compiler<'src, 'c> {
    scanner:    Scanner<'src>,
    parser:     Parser<'src>,
    chunk:      &'c mut Chunk,
}

#[repr(u8)]
#[derive(FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment,     // =
    Or,             // or
    And,            // and
    Equality,       // == !=
    Comparison,     // < > <= >=
    Term,           // + -
    Factor,         // * /
    Unary,          // ! -
    Call,           // . ()
    Primary,
}

impl Precedence {
    fn up(self) -> Self {
        Precedence::from_u8(self as u8 + 1).unwrap_or(Precedence::Primary)
    }
}

struct ParseRule<'src, 'c> {
    prefix:     Option<fn(&mut Compiler<'src, 'c>)>,
    infix:      Option<fn(&mut Compiler<'src, 'c>)>,
    precedence: Precedence
}

impl<'src, 'c> Compiler<'src, 'c> {

    fn error_at_current(&mut self, msg: &str) {
        let cur = self.parser.current;
        self.error_at(&cur, msg);
    }

    fn error(&mut self, msg: &str) {
        let prev = self.parser.previous;
        self.error_at(&prev, msg);
    }

    fn error_at(&mut self, tok: &Token, msg: &str) {
        if self.parser.panic_mode { return; }
        self.parser.panic_mode = true;

        print!("[line {}] Error", tok.line);

        match tok.ty {
            TokenType::EndFile => print!(" at end"),
            TokenType::Error => {},
            _ => print!(" at '{}'", tok.slice)
        }

        print!(": {}\n", msg);
        self.parser.had_error = true;
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.ty != TokenType::Error { break; }

            let slc = self.parser.current.slice;
            self.error_at_current(slc);
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &str) {
        if self.parser.current.ty == ty {
            self.advance();
            return;
        }

        self.error_at_current(msg);
    }
    
    fn emit_byte<T: Into<u8>>(&mut self, byte: T) {
        self.chunk.write_code(byte, self.parser.previous.line as u32);
    }

    fn emit_bytes<T: Into<u8>>(&mut self, one: T, two: T) {
        self.emit_byte(one);
        self.emit_byte(two);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OPCODE::Return);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if DEBUG_PRINT_CODE {
            if !self.parser.had_error {
                dissassemble_chunk(&self.chunk, "code");
            }
        }
    }

    fn get_rule(&self, ty: TokenType) -> ParseRule<'src, 'c> {
        match ty {
            TokenType::LeftParen => ParseRule { prefix: Some(Compiler::grouping), infix: None, precedence: Precedence::None },
            TokenType::RightParen => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::LeftBrace => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::RightBrace => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Comma => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Dot => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Minus => ParseRule { prefix: Some(Compiler::unary), infix: Some(Compiler::binary), precedence: Precedence::Term },
            TokenType::Plus => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Term },
            TokenType::Semicolon => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Slash => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Factor },
            TokenType::Star => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Factor },
            TokenType::Bang => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::BangEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Equal => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::EqualEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Greater => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::GreaterEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Less => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::LessEqual => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Identifier => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Str => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Num => ParseRule { prefix: Some(Compiler::number), infix: None, precedence: Precedence::None },
            TokenType::And => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Class => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Else => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::False => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::For => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Fun => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::If => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Nil => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Or => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Print => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Return => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Super => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::This => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::True => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Var => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::While => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Error => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::EndFile => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        }
    }

    fn parse_precendence(&mut self, prec: Precedence) {
        self.advance();
        if let Some(prefix_rule) = self.get_rule(self.parser.previous.ty).prefix {
            prefix_rule(self);
        }
        else {
            self.error("Expect expression.");
        }

        while prec <= self.get_rule(self.parser.current.ty).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.parser.previous.ty).infix.unwrap();
            infix_rule(self);
        }
    }

    fn binary(&mut self) {
        let op_ty = self.parser.previous.ty;
        let rule = self.get_rule(op_ty);
        self.parse_precendence(rule.precedence.up());

        match op_ty {
            TokenType::Plus => self.emit_byte(OPCODE::Add),
            TokenType::Minus => self.emit_byte(OPCODE::Subtract),
            TokenType::Star => self.emit_byte(OPCODE::Multiply),
            TokenType::Slash => self.emit_byte(OPCODE::Divide),
            _ => unreachable!()
        }
    }

    fn make_constant(&mut self, v: &Value) -> u8 {
        let x = self.chunk.add_constant(*v);
        if x > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        x as u8
    }

    fn emit_constant(&mut self, v: &Value) {
        let two = self.make_constant(v);
        self.emit_bytes(OPCODE::Constant.into(), two);
    }

    fn number(&mut self) {
        self.emit_constant(&self.parser.previous.slice.parse().unwrap());
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_ty = self.parser.previous.ty;

        self.parse_precendence(Precedence::Unary);

        match operator_ty {
            TokenType::Minus => self.emit_byte(OPCODE::Negate),
            _ => return
        }
    }

    fn expression(&mut self) {
        self.parse_precendence(Precedence::Assignment);
    }

    pub fn compile(source: &str) -> Result<Chunk, IError> {
        let mut chunk = Chunk::default();
        let mut c = Compiler {
            scanner: Scanner::new(source),
            parser: Parser {
                current: Token { slice: "", ty: TokenType::Nil, line: 67 },
                previous: Token { slice: "", ty: TokenType::Nil, line: 67 },
                had_error: false,
                panic_mode: false,
            },
            chunk: &mut chunk,
        };

        c.advance();
        c.expression();
        c.consume(TokenType::EndFile, "Expect end of expression.");
        c.end_compiler();

        if c.parser.had_error {
            return Err(IError::CompileError)
        }

        Ok(chunk)
    }
}