use crate::frontend::scanner::*;

#[derive(Default)]
pub struct Compiler {}

impl Compiler {
    pub fn compile(&self, source: &str) {
        let mut scanner = Scanner::default();
        let mut line = -1;

        loop {
            let token: Token = scanner.scan_token();
            if token.line != line {
                print!("{:4}", token.line);
                line = token.line;
            }
            else {
                print!("   |");
            }
            println!("{:2} '{}'", token.ty, token.text);

            if token.ty == TokenType::EOF { break; }
        }
    }
}