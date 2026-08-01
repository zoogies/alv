use std::io;
use std::process::exit;

use alv_vm::chunk::*;
use alv_vm::debug::*;
use alv_vm::value::*;
use alv_vm::backend::vm::*;

use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(name= "alv")]
struct Cli {
    /// Path to the .alv source file
    input: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.input {
        Some(input) => run_file(&input),
        None => repl()
    } // clap handles "usage" for us on it's own
}

fn run_file(path: &str) {
    let program = match std::fs::read_to_string(path) {
        Ok(program) => program,
        Err(error) => {
            exit(-1); // TODO: something better
        }
    };
    let mut vm = VM::default();
    let res = vm.interpret(input);

    match res {
        Ok(()) => exit(0),
        Err(IError::CompileError) => exit(65),
        Err(IError::RuntimeError) => exit(70),
    }
}

fn repl() {
    let mut input = String::new();
    loop {
        print!("> ");

        io::stdin().read_line(&mut input).expect("Failed to read line");
        println!();

        let mut vm = VM::default();
        let res = vm.interpret(&input);
    }
}
