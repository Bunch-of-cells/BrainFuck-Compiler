use brainfuck::*;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("\x1b[1mParsing arguments...\x1b[0m");
    let args = match parse_args(&args) {
        Ok(res) => {
            if res.console {
                println!("\x1b[1mRunning brainfuck interpreter in console...\x1b[0m");
                match run_in_terminal(res.debug && !res.release, res.mem_size, res.offset) {
                    Ok(_) => process::exit(0),
                    Err(e) => {
                        eprintln!("\x1b[91mError\x1b[0m: {}", e);
                        process::exit(1);
                    }
                }
            }
            res
        }
        Err(err) => {
            eprintln!("\x1b[91mError\x1b[0m: {}", err);
            process::exit(1);
        }
    };

    if args.file.is_empty() {
        process::exit(0)
    }

    println!("\x1b[1mGetting file contents from {}...\x1b[0m", args.file);

    let contents = match get_code(args.file) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("\x1b[91mError\x1b[0m: {}", e);
            process::exit(1);
        }
    };

    if args.interpret {
        println!("\x1b[1mInterpreting the code...\x1b[0m");
        match interpret(
            &contents,
            args.debug && !args.release,
            args.mem_size,
            args.offset,
        ) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("\x1b[91mError\x1b[0m: {}", e);
                process::exit(1);
            }
        }
    } else {
        println!("\x1b[1mTranspiling the code to C...\x1b[0m");
        match compile(&contents, args) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("\x1b[91mError\x1b[0m: {}", e);
                process::exit(1);
            }
        }
    }
}
