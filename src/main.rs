mod lexer;
mod log;
mod parser;
mod treewalk;

use std::env;
use std::process::ExitCode;

use crate::log::alv_error;
use crate::log::alv_log;
use crate::treewalk::TWInterp;

use lexer::Literal;
use lexer::Token;
use lexer::TokenType;

use parser::Parser;
use parser::Expr;

fn print_usage() {
    println!("usage: alv <input file>.alv");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    // for now just take one file as input
    if argc < 2 || argc >= 3 {
        print_usage();
        return ExitCode::FAILURE
    }

    let input_path = &args[1];

    let program = match std::fs::read_to_string(input_path) {
        Ok(program) => program,
        Err(error) => {
            alv_error!("failed to read '{}': {}", input_path, error);
            return ExitCode::FAILURE;
        }
    };

    alv_log!("-------------   Input Program    -------------");
    println!("{}\n", program);
    alv_log!("-------------    Lexer Output    -------------");

    let l = lexer::Lexer::new(&program);
    
    let tokens = l.scan_tokens();

    alv_log!("tokens length: {}", tokens.len());
    for tok in &tokens {
        alv_log!("tok {:?}",tok);
    }

    let mut p = Parser::new(tokens);
    let e = p.parse();
    match e {
        Ok(e) => {
            // println!("\n");
            // alv_log!("-------------   Parser Output    -------------");
            // println!("{:#?}", e);

            println!("\n");
            alv_log!("------------- Interpreter Output -------------");
            
            // TODO: should exit with code 70 if runtime error occurred
            let mut i = TWInterp::new();
            i.interpret(&e)
        },
        Err(_e) => {
            std::process::exit(65);
        }
    }
}
