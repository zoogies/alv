mod lexer;

use std::env;
use std::process::ExitCode;

// use lexer::Lexer;

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
            eprintln!("failed to read '{}': {}", input_path, error);
            return ExitCode::FAILURE;
        }
    };

    println!("input program:\n{}", program);

    return ExitCode::SUCCESS
}
