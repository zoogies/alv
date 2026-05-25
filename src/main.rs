use std::env;
use std::process::ExitCode;

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

    return ExitCode::SUCCESS
}
