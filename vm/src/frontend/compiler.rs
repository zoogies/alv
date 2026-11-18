use num_derive::FromPrimitive;
use num_traits::FromPrimitive as _;

use crate::common::*;
use crate::debug::dissassemble_chunk;
use crate::frontend::scanner::*;
use crate::chunk::*;
use crate::backend::vm::*;
use crate::value::*;
use crate::heap::*;

struct Parser<'src> {
    current:    Token<'src>, 
    previous:   Token<'src>, 
    had_error:  bool,
    panic_mode: bool,
}

pub struct Local<'src> {
    name: &'src str,    // no need for token because we report errors using parser lineno
    depth: Option<u32>, // may or may not be initialized. u32 is arbitrary, needs to be large for edge case weird depths {{{{{...{x}...}}}}}
}

pub struct FunctionState<'src> {
    function:       Function,
    fn_type:        FunctionType,
    locals:         Vec<Local<'src>>,
    scope_depth:    u32,
}

pub struct Compiler<'src, 'c> {
    scanner:    Scanner<'src>,
    parser:     Parser<'src>,
    heap:       &'c mut Heap,
    states:     Vec<FunctionState<'src>>,
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
    prefix:     Option<fn(&mut Compiler<'src, 'c>, bool)>,
    infix:      Option<fn(&mut Compiler<'src, 'c>, bool)>,
    precedence: Precedence
}

impl<'src, 'c> Compiler<'src, 'c> {

    fn current(&mut self) -> &mut FunctionState<'src> {
        self.states.last_mut().expect("Tried to access current() when there are zero states.")
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.current().function.chunk
    }

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
    
    fn match_token(&mut self, ty: TokenType) -> bool {
        if !self.check(ty) { return false };
        self.advance();
        true
    }

    fn check(&self, ty: TokenType) -> bool {
        self.parser.current.ty == ty
    }

    fn emit_byte<T: Into<u8>>(&mut self, byte: T) {
        let pl = self.parser.previous.line;
        self.current_chunk().write_code(byte, pl as u32);
    }

    fn emit_bytes<T: Into<u8>>(&mut self, one: T, two: T) {
        self.emit_byte(one);
        self.emit_byte(two);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OPCODE::Return);
    }

    fn end_compiler(&mut self) -> ObjRef {
        self.emit_return();

        let state = self.states.pop().expect("Ended compilation without a single state.");

        if DEBUG_PRINT_CODE {
            if !self.parser.had_error {
                dissassemble_chunk(
                    &state.function.chunk,
                    state.function.name.as_deref().unwrap_or("<script>"),
                    self.heap
                );
            }
        }

        self.heap.alloc(Obj::Function(state.function))
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
            TokenType::Bang => ParseRule { prefix: Some(Compiler::unary), infix: None, precedence: Precedence::None },
            TokenType::BangEqual => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Equality },
            TokenType::Equal => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::EqualEqual => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Equality },
            TokenType::Greater => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Comparison },
            TokenType::GreaterEqual => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Comparison },
            TokenType::Less => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Comparison },
            TokenType::LessEqual => ParseRule { prefix: None, infix: Some(Compiler::binary), precedence: Precedence::Comparison },
            TokenType::Identifier => ParseRule { prefix: Some(Compiler::variable), infix: None, precedence: Precedence::None },
            TokenType::Str => ParseRule { prefix: Some(Compiler::string), infix: None, precedence: Precedence::None },
            TokenType::Num => ParseRule { prefix: Some(Compiler::number), infix: None, precedence: Precedence::None },
            TokenType::And => ParseRule { prefix: None, infix: Some(Compiler::and), precedence: Precedence::And },
            TokenType::Class => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Else => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::False => ParseRule { prefix: Some(Compiler::literal), infix: None, precedence: Precedence::None },
            TokenType::For => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Fun => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::If => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Nil => ParseRule { prefix: Some(Compiler::literal), infix: None, precedence: Precedence::None },
            TokenType::Or => ParseRule { prefix: None, infix: Some(Compiler::or), precedence: Precedence::Or },
            TokenType::Print => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Return => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Super => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::This => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::True => ParseRule { prefix: Some(Compiler::literal), infix: None, precedence: Precedence::None },
            TokenType::Var => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::While => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Error => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::EndFile => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
        }
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();
        let can_assign = prec <= Precedence::Assignment;

        if let Some(prefix_rule) = self.get_rule(self.parser.previous.ty).prefix {
            prefix_rule(self,can_assign);
        
            while prec <= self.get_rule(self.parser.current.ty).precedence {
                self.advance();
                let infix_rule = self.get_rule(self.parser.previous.ty).infix.unwrap();
                infix_rule(self, can_assign);
            }

            if can_assign && self.match_token(TokenType::Equal) {
                self.error("Invalid assignment target.");
            }
        }
        else {
            self.error("Expect expression.");
            return;
        }
    }
    
    fn mark_initialized(&mut self) {
        self.current().locals.last_mut().unwrap().depth = Some(self.current().scope_depth);
    }
    
    fn parse_variable(&mut self, msg: &str) -> u8 {
        self.consume(TokenType::Identifier, msg);

        self.declare_variable();
        if self.current().scope_depth > 0 { return 0 }

        self.identifier_constant(self.parser.previous.slice)
    }

    fn define_variable(&mut self, global: u8) {
        if self.current().scope_depth > 0 {
            self.mark_initialized();
            return
        }
        
        self.emit_byte(OPCODE::DefineGlobal);
        self.emit_byte(global);
    }

    fn declare_variable(&mut self) {
        if self.current().scope_depth == 0 { return }

        let name = self.parser.previous.slice;
        let sd = self.current().scope_depth;

        let dup = self.current().locals
            .iter()
            .rev()
            .take_while(|l| l.depth.is_some_and(|l| l == sd ))
            .any(|l| l.name == name);

        if dup {
            self.error("Already a variable with this name in this scope.");
        }

        self.add_local(name);
    }

    fn add_local(&mut self, name: &'src str) {
        if self.current().locals.len() == u8::MAX as usize + 1 {
            self.error("Too many local variables in function.");
            return;
        }

        self.current().locals.push(Local {
            depth: None,
            name: name,
        });
    }

    fn identifier_constant(&mut self, name: &str) -> u8 {
        let c = self.heap.alloc_string(name);
        self.make_constant(&Value::Obj(c))
    }

    fn binary(&mut self, _can_assign: bool) {
        let op_ty = self.parser.previous.ty;
        let rule = self.get_rule(op_ty);
        self.parse_precedence(rule.precedence.up());

        match op_ty {
            TokenType::Plus => self.emit_byte(OPCODE::Add),
            TokenType::Minus => self.emit_byte(OPCODE::Subtract),
            TokenType::Star => self.emit_byte(OPCODE::Multiply),
            TokenType::Slash => self.emit_byte(OPCODE::Divide),
            TokenType::BangEqual => self.emit_bytes(OPCODE::Equal, OPCODE::Not),
            TokenType::EqualEqual => self.emit_byte(OPCODE::Equal),
            TokenType::Greater => self.emit_byte(OPCODE::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OPCODE::Less, OPCODE::Not),
            TokenType::Less => self.emit_byte(OPCODE::Less),
            TokenType::LessEqual => self.emit_bytes(OPCODE::Greater, OPCODE::Not),
            _ => unreachable!()
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.parser.previous.ty {
            TokenType::False => self.emit_byte(OPCODE::False),
            TokenType::Nil => self.emit_byte(OPCODE::Nil),
            TokenType::True => self.emit_byte(OPCODE::True),
            _ => { unreachable!() }
        }
    }

    fn string(&mut self, _can_assign: bool) {       
        let r = self.heap.alloc_string(
            &self.parser.previous.slice[1..self.parser.previous.slice.len() - 1]
        );
        
        self.emit_constant(&Value::Obj(r));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.slice, can_assign);
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        let i = self.current().locals
            .iter()
            .rposition(|x| x.name == name)?;

        if self.current().locals[i].depth.is_none() {
            self.error("Can't read local variable in its own initializer.");
        }

        Some(i as u8)
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let (get_op, set_op, arg) = match self.resolve_local(name) {
            Some(slot) => (OPCODE::GetLocal, OPCODE::SetLocal, slot),
            None => (OPCODE::GetGlobal, OPCODE::SetGlobal, self.identifier_constant(name))
        };

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_byte(set_op);
            self.emit_byte(arg);
        }
        else {
            self.emit_byte(get_op);
            self.emit_byte(arg);
        }
    }

    fn make_constant(&mut self, v: &Value) -> u8 {
        let x = self.current_chunk().add_constant(*v);
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

    fn number(&mut self, _can_assign: bool) {
        let val: f64 = self.parser.previous.slice.parse().unwrap();
        self.emit_constant(&Value::Number(val));
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OPCODE::JumpIfFalse as u8);
        self.emit_byte(OPCODE::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(OPCODE::JumpIfFalse as u8);
        let end_jump = self.emit_jump(OPCODE::Jump as u8);
        
        self.patch_jump(else_jump);
        self.emit_byte(OPCODE::Pop as u8);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_ty = self.parser.previous.ty;

        self.parse_precedence(Precedence::Unary);

        match operator_ty {
            TokenType::Minus => self.emit_byte(OPCODE::Negate),
            TokenType::Bang => self.emit_byte(OPCODE::Not),
            _ => return
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn var_declaration(&mut self) {
        let global: u8 = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        }
        else {
            self.emit_byte(OPCODE::Nil);
        }

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(global);
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_declaration();
        }
        else {
            self.statement();
        }

        if self.parser.panic_mode { self.synchronize(); }
    }

    fn begin_scope(&mut self) {
        self.current().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.current().scope_depth -= 1;
        let sd = self.current().scope_depth;

        while self.current().locals.last().is_some_and(|l| l.depth.is_some_and(|d| d > sd)) {
            self.current().locals.pop();
            self.emit_byte(OPCODE::Pop);
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EndFile) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        }
        else if self.match_token(TokenType::For) {
            self.for_statement();
        }
        else if self.match_token(TokenType::If) {
            self.if_statement();
        }
        else if self.match_token(TokenType::While) {
            self.while_statement();
        }
        else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        }
        else {
            self.expression_statement();
        }
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after 'while'.");
        
        let exit_jump = self.emit_jump(OPCODE::JumpIfFalse as u8);
        self.emit_byte(OPCODE::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OPCODE::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope(); // iterator var scope

        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.match_token(TokenType::Semicolon) {} // optional
        else if self.match_token(TokenType::Var) { // declare iterator var
            self.var_declaration();
        }
        else {
            self.expression_statement();
        }
    
        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump: isize = -1; // TODO: could use Option but lazy translation
        if !self.match_token(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            exit_jump = self.emit_jump(OPCODE::JumpIfFalse as u8) as isize;
            self.emit_byte(OPCODE::Pop);
        }

        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(OPCODE::Jump as u8);
            let increment_start = self.current_chunk().code.len();
            self.expression();
            self.emit_byte(OPCODE::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");
            
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_byte(OPCODE::Pop);
        }

        self.end_scope(); // iterator var scope
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
    
        let then_jump = self.emit_jump(OPCODE::JumpIfFalse as u8);
        self.emit_byte(OPCODE::Pop);
        self.statement();

        let else_jump = self.emit_jump(OPCODE::Jump as u8);

        self.patch_jump(then_jump);
        self.emit_byte(OPCODE::Pop);
    
        if self.match_token(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OPCODE::Loop);

        let offset = self.current_chunk().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_byte((offset >> 8) as u8);
        self.emit_byte(offset as u8);
    }

    fn emit_jump(&mut self, instr: u8) -> usize {
        self.emit_byte(instr);
        self.emit_byte(0xff);
        self.emit_byte(0xff);

        self.current_chunk().code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().code.len() - offset - 2;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        self.current_chunk().code[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OPCODE::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OPCODE::Pop);
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        while self.parser.current.ty != TokenType::EndFile {
            if self.parser.previous.ty == TokenType::Semicolon { return; }
            match self.parser.current.ty {
                TokenType::Class |
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }

    fn init_compiler(&mut self, fn_type: FunctionType) {
        self.states.push(FunctionState {
            fn_type,
            function: Function::default(),
            locals: vec![Local {
                name: "",
                depth: Some(0),
            }],
            scope_depth: 0,
        });
    }

    pub fn compile(source: &str, heap: &mut Heap) -> Result<ObjRef, IError> {
        let mut c = Compiler {
            scanner: Scanner::new(source),
            parser: Parser {
                current: Token { slice: "", ty: TokenType::Nil, line: 67 },
                previous: Token { slice: "", ty: TokenType::Nil, line: 67 },
                had_error: false,
                panic_mode: false,
            },
            heap: heap,
            states: Vec::new(),
        };

        c.init_compiler(FunctionType::Script);

        c.advance();
        
        while !c.match_token(TokenType::EndFile) {
            c.declaration();
        }

        let r = c.end_compiler();

        if c.parser.had_error {
            return Err(IError::CompileError)
        }

        Ok(r)
    }
}