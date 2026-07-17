mod codegen;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: skyscraper [-o output] <file.sky>");
        process::exit(1);
    }

    let mut output: Option<PathBuf> = None;
    let mut input: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: -o requires an argument");
                    process::exit(1);
                }
                output = Some(PathBuf::from(&args[i]));
            }
            other => {
                if other.starts_with('-') {
                    eprintln!("Error: unknown option: {}", other);
                    process::exit(1);
                }
                input = Some(other.to_string());
            }
        }
        i += 1;
    }

    let filename = match input {
        Some(f) => f,
        None => {
            eprintln!("Usage: skyscraper [-o output] <file.sky>");
            process::exit(1);
        }
    };

    if !filename.ends_with(".sky") {
        eprintln!("Error: expected .sky file, got: {}", filename);
        process::exit(1);
    }

    let source = match fs::read_to_string(&filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", filename, e);
            process::exit(1);
        }
    };

    let mut parser = match parser::Parser::new(&source, &filename) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Lexer error at {}:{}: {}", filename, e.line, e.message);
            process::exit(1);
        }
    };

    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error at {}:{}: {}", e.file, e.line, e.message);
            process::exit(1);
        }
    };

    let out_path = output.unwrap_or_else(|| {
        let stem = std::path::Path::new(&filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        PathBuf::from(stem)
    });

    match codegen::compile(&program, &out_path) {
        Ok(()) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&out_path, fs::Permissions::from_mode(0o755));
            }
            println!("Assembled to {}", out_path.display());
        }
        Err(e) => {
            eprintln!("Codegen error: {}", e);
            process::exit(1);
        }
    }
}
