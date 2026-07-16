mod codegen;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: skyscraper <file.sky>");
        process::exit(1);
    }

    let filename = &args[1];

    if !filename.ends_with(".sky") {
        eprintln!("Error: expected .sky file, got: {}", filename);
        process::exit(1);
    }

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", filename, e);
            process::exit(1);
        }
    };

    let mut parser = match parser::Parser::new(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Lexer error at {}:{}: {}", filename, e.line, e.message);
            process::exit(1);
        }
    };

    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error at {}:{}: {}", filename, e.line, e.message);
            process::exit(1);
        }
    };

    println!("Parsed {} statements", program.statements.len());

    for stmt in &program.statements {
        println!("  {:?}", stmt);
    }
}
