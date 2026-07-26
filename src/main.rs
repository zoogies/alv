use alv::frontend::{lexer, parser::Parser, resolver::Resolver};
use alv::backend::treewalk::TWInterp;

use alv::{alv_error, alv_log};

use std::process::ExitCode;

use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(name= "alv")]
struct Cli {
    /// Path to the .alv source file
    input: String,
    
    /// Print the input program
    #[arg(long)]
    print_program: bool,

    /// Dump the lexer token stream
    #[arg(long)]
    print_tokens: bool,
    
    /// Dump the AST after parsing
    #[arg(long)]
    print_ast: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let program = match std::fs::read_to_string(&cli.input) {
        Ok(program) => program,
        Err(error) => {
            alv_error!("failed to read '{}': {}", cli.input, error);
            return ExitCode::FAILURE;
        }
    };

    if cli.print_program {
        alv_log!("-------------   Input Program    -------------");
        println!("{}", program);
        println!("\n");
    }

    let l = lexer::Lexer::new(&program);
    let tokens = l.scan_tokens();

    if cli.print_tokens {
        alv_log!("-------------    Lexer Output    -------------");
        alv_log!("tokens length: {}", tokens.len());
        for tok in &tokens {
            alv_log!("tok {:?}",tok);
        }
    }

    let mut p = Parser::new(tokens);
    match p.parse() {
        Ok(e) => {            
            if cli.print_ast {
                println!("\n");
                alv_log!("-------------   Parser Output    -------------");
                println!("{:#?}", e);
            }

            println!("\n");
            alv_log!("------------- Interpreter Output -------------");
            
            let mut i = TWInterp::new(&p.id_counter);
            
            let mut r = Resolver::new(&mut i);
            if r.resolve(&e) {
                std::process::exit(65);
            }
            
            // exit with code 70 if runtime error occurred, per bob
            if i.interpret(&e) != ExitCode::SUCCESS {
                std::process::exit(70);
            }

            ExitCode::SUCCESS
        },
        Err(_e) => {
            std::process::exit(65);
        }
    }
}
