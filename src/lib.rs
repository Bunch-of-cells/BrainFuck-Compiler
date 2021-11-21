use getch::Getch;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, Write},
    process::Command,
};

pub struct Args<'a> {
    pub console: bool,
    pub interpret: bool,
    pub file: &'a str,
    pub debug: bool,
    pub mem_size: usize,
    pub offset: usize,
    pub release: bool,
    run: bool,
    output: &'a str,
    keep: bool,
    compiler: &'a str,
}

struct ArgFlags(u16);

impl ArgFlags {
    const HELP: u16 = 1;
    const FILE: u16 = 2;
    const OUTPUT: u16 = 4;
    const KEEP: u16 = 8;
    const COMPILER: u16 = 16;
    const RUN: u16 = 32;
    const INTERPRET: u16 = 64;
    const DEBUG: u16 = 128;
    const MEM_SIZE: u16 = 256;
    const OFFSET: u16 = 512;
    const RELEASE: u16 = 1024;
}

impl Default for Args<'_> {
    fn default() -> Self {
        Self {
            release: false,
            offset: 0,
            mem_size: 30000,
            console: false,
            run: false,
            file: "",
            output: "output",
            keep: false,
            compiler: "gcc",
            interpret: false,
            debug: false,
        }
    }
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut parsed_args = Args::default();
    let mut flags = ArgFlags(0);

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                if flags.0 & ArgFlags::HELP != 0 {
                    continue;
                }
                flags.0 |= ArgFlags::HELP;
                println!("\nUsage:\n  brainfuck [options] file\n");
                println!("Options:");
                println!("  --help | -h            Show this message");
                println!("  --keep | -k            Keep the generated C file");
                println!(
                    "  --output | -o          Set the name of the output file, default is 'output'"
                );
                println!("  --compiler | -c        Select the compiler to compile the C generated file, default is gcc");
                println!("  --run | -r             Runs the program after compiling");
                println!("  --interpret | -i       Interprets the program instead of compiling it");
                println!("  --debug | -d           Activates the debug mode.\n\t\t\t In the debug mode, any # will be considered as a debug symbol");
                println!("  --mem_size | -m        Set the memory, default is 30000");
                println!("  --release | -rl        Compiles in release mode");
                println!("  --ptr-offset | -po     Set the pointer offset from the start of the memory, default is 0\n");
            }
            "--keep" | "-k" => {
                if flags.0 & ArgFlags::KEEP != 0 {
                    return Err("More than 1 build flag passed".to_owned());
                }
                flags.0 |= ArgFlags::KEEP;
                parsed_args.keep = true;
            }
            "--release" | "-rl" => {
                if flags.0 & ArgFlags::RELEASE != 0 {
                    return Err("More than 1 release flag passed".to_owned());
                }
                flags.0 |= ArgFlags::RELEASE;
                parsed_args.release = true;
            }
            "--debug" | "-d" => {
                if flags.0 & ArgFlags::DEBUG != 0 {
                    return Err("More than 1 debug flag passed".to_owned());
                }
                flags.0 |= ArgFlags::DEBUG;
                parsed_args.debug = true;
            }
            "--interpret" | "-i" => {
                if flags.0 & ArgFlags::INTERPRET != 0 {
                    return Err("More than 1 interpret flag passed".to_owned());
                }
                flags.0 |= ArgFlags::INTERPRET;
                parsed_args.interpret = true;
            }
            "--run" | "-r" => {
                if flags.0 & ArgFlags::RUN != 0 {
                    return Err("More than 1 run flag passed".to_owned());
                }
                flags.0 |= ArgFlags::RUN;
                parsed_args.run = true;
            }
            other => match other.split_once('=') {
                Some(("--output" | "-o", var)) => {
                    if flags.0 & ArgFlags::OUTPUT != 0 {
                        return Err("More than 1 output flag passed".to_owned());
                    }
                    flags.0 |= ArgFlags::OUTPUT;
                    parsed_args.output = var
                }
                Some(("--mem-size" | "-m", var)) => {
                    if flags.0 & ArgFlags::MEM_SIZE != 0 {
                        return Err("More than 1 output flag passed".to_owned());
                    }
                    flags.0 |= ArgFlags::MEM_SIZE;
                    parsed_args.mem_size = match var.parse::<usize>() {
                        Ok(res) => res,
                        Err(err) => return Err(err.to_string()),
                    }
                }
                Some(("--ptr-offset" | "-po", var)) => {
                    if flags.0 & ArgFlags::OFFSET != 0 {
                        return Err("More than 1 output flag passed".to_owned());
                    }
                    flags.0 |= ArgFlags::OFFSET;
                    parsed_args.offset = match var.parse::<usize>() {
                        Ok(res) => res,
                        Err(err) => return Err(err.to_string()),
                    }
                }
                Some(("--compiler" | "-c", var)) => {
                    if flags.0 & ArgFlags::COMPILER != 0 {
                        return Err("More than 1 compiler flag passed".to_owned());
                    }
                    flags.0 |= ArgFlags::COMPILER;
                    parsed_args.compiler = var
                }
                None => {
                    if flags.0 & ArgFlags::FILE != 0 {
                        return Err("More than 1 file passed".to_owned());
                    }
                    if !other.ends_with(".bf") {
                        return Err("File must end with .bf".to_owned());
                    }
                    flags.0 |= ArgFlags::FILE;
                    parsed_args.file = other;
                }
                Some(_) => {
                    return Err(format!("Invalid argument {}", other));
                }
            },
        }
    }
    validate_args(flags, parsed_args)
}

fn validate_args(flags: ArgFlags, mut args: Args) -> Result<Args, String> {
    if args.offset > args.mem_size {
        return Err("pointer offset cannot be greater than memory size".to_owned());
    }
    if flags.0 & 896 != 0 || flags.0 == 0 {
        args.console = true;
        return Ok(args);
    }
    if flags.0 & 1276 != 0 && flags.0 & 2 == 0 {
        return Err("No File passed".to_owned());
    }
    Ok(args)
}

pub fn get_code(filename: &str) -> Result<String, String> {
    let mut contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => return Err(err.to_string()),
    };

    contents.retain(|c| "<>[]+-.,#".contains(c));
    if contents.matches('[').count() != contents.matches(']').count() {
        return Err("Unbalanced Brackets".to_string());
    }
    Ok(contents)
}

pub fn interpret(
    contents: &str,
    debug: bool,
    mem_size: usize,
    ptr_offset: usize,
) -> Result<(), &str> {
    let mut mem = vec![0];
    for _ in 0..ptr_offset {
        mem.push(0)
    }
    let mut cellptr = ptr_offset;
    let mut debug_count = 0;
    let getch = Getch::new().unwrap();
    let mut codeptr = 0;
    let mut bracemap: HashMap<usize, usize> = HashMap::new();
    let mut temp = Vec::new();

    for (pos, code) in contents.chars().enumerate() {
        if code == '[' {
            temp.push(pos)
        } else if code == ']' {
            let start = temp.pop().unwrap();
            bracemap.insert(start, pos);
            bracemap.insert(pos, start);
        }
    }
    println!("\n\x1b[90m--------------\x1b[0m\x1b[96mOUTPUT\x1b[0m\x1b[90m--------------\x1b[0m\n");
    while codeptr < contents.len() {
        let code = contents.chars().nth(codeptr).unwrap();
        match code {
            '>' => {
                cellptr += 1;
                if cellptr > mem_size {
                    return Err("Memory index out of bound");
                }
                if cellptr == mem.len() {
                    mem.push(0)
                }
            }
            '<' => {
                if cellptr == 0 {
                    return Err("Memory index out of bound");
                }
                cellptr -= 1;
            }
            '+' => mem[cellptr] += 1,
            '-' => mem[cellptr] -= 1,
            '.' => print!("{}", (mem[cellptr]) as char),
            ',' => mem[cellptr] = getch.getch().unwrap(),
            '[' => {
                if mem[cellptr] == 0 {
                    codeptr = *bracemap.get(&codeptr).unwrap()
                }
            }
            ']' => {
                if mem[cellptr] != 0 {
                    codeptr = *bracemap.get(&codeptr).unwrap()
                }
            }
            '#' if debug => {
                debug_count += 1;

                println!(
                    "\ndebug flag {} : {} {} {}",
                    debug_count, mem[cellptr] as char, mem[cellptr], cellptr
                )
            }
            _ => {}
        }
        codeptr += 1;
    }
    println!("\n\x1b[90m----------------------------------\x1b[0m");
    Ok(())
}

fn translate(contents: &str, debug: bool, mem: usize, offset: usize) -> String {
    let mut cpp_code = format!(
        "\
#include <stdio.h>
#include <termios.h>

int getch() {{
    struct termios old, new;
    int ch;
    tcgetattr(0, &old);
    new = old;
    new.c_lflag &= ~ICANON;
    new.c_lflag &= ~ECHO;
    tcsetattr(0, TCSANOW, &new);
    ch = getchar();
    tcsetattr(0, TCSANOW, &old);
    return ch;
}}

int main() {{
    char mem[{}];
    char* ptr = mem + {};

",
        mem, offset
    );
    if debug {
        cpp_code.push_str("\tunsigned int debug_count = 0;\n")
    }
    for code in contents.chars() {
        cpp_code.push_str(match code {
            '>' => "\tptr++;\n",
            '<' => "\tptr--;\n",
            '+' => "\t(*ptr)++;\n",
            '-' => "\t(*ptr)--;\n",
            '.' => "\tprintf(\"%c\", *ptr);\n",
            ',' => "\t*ptr = getch();\n",
            '[' => "\twhile (*ptr) {\n",
            ']' => "\t}\n",
            '#' if debug => {
                "\tdebug_count += 1;printf(\"\\ndebug flag %d : %c, %d, %ld\\n\", debug_count, *ptr, *ptr, ptr-mem);"
            }
            _ => "",
        })
    }
    cpp_code.push_str("\treturn 0;\n}\n");
    cpp_code
}

pub fn compile(contents: &str, args: Args) -> Result<(), Box<dyn Error>> {
    println!("\x1b[1mCreating the C file...\x1b[0m");
    let mut cpp_file = File::create([args.output, ".c"].concat())?;
    cpp_file.write_all(
        translate(
            contents,
            args.debug && !args.release,
            args.mem_size,
            args.offset,
        )
        .as_bytes(),
    )?;

    println!(
        "\x1b[1mCompiling the C file using {}...\x1b[0m",
        args.compiler
    );
    let output = Command::new(args.compiler)
        .args([[args.output, ".c"].concat().as_str(), "-o", args.output])
        .output()?;

    for x in output.stderr.iter() {
        print!("{}", *x as char)
    }

    if !args.keep {
        println!("\x1b[1mDeleting the C file...\x1b[0m");
        fs::remove_file([args.output, ".c"].concat())?
    }
    if args.run {
        run(args.output)?
    }
    Ok(())
}

fn run(filename: &str) -> Result<(), Box<dyn Error>> {
    println!("\x1b[1mRunning the program...\x1b[0m");
    let program = Command::new(format!("./{}", filename)).output()?;

    println!("\n\x1b[90m--------------\x1b[0m\x1b[96mSTDOUT\x1b[0m\x1b[90m--------------\x1b[0m\n");
    for x in program.stdout.iter() {
        print!("{}", *x as char)
    }

    if !program.stderr.is_empty() {
        println!(
            "\n\x1b[90m--------------\x1b[0m\x1b[96mSTDERR\x1b[0m\x1b[90m--------------\x1b[90m\n"
        );
        for x in program.stderr.iter() {
            print!("\x1b[91m{}\x1b[0m", *x as char)
        }
    }

    println!("\n\x1b[90m----------------------------------\x1b[0m");
    if program.status.success() {
        println!(
            "\x1b[1mProgram ended with \x1b[0m\x1b[92m{}\x1b[0m",
            program.status
        );
    } else {
        println!(
            "\x1b[1mProgram ended with \x1b[90m\x1b[91m{}\x1b[0m",
            program.status
        );
    }
    Ok(())
}

pub fn run_in_terminal(
    debug: bool,
    mem_size: usize,
    ptr_offset: usize,
) -> Result<(), &'static str> {
    let getch = Getch::new().unwrap();
    let mut mem = vec![0];
    for _ in 0..ptr_offset {
        mem.push(0)
    }
    let mut cellptr = ptr_offset;
    let mut contents = String::new();
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut contents).unwrap();
        contents.pop();
        println!("{}", contents);
        if contents.eq("quit") {
            break Ok(());
        }
        eval(&contents, &mut mem, &mut cellptr, debug, &getch, mem_size)?;
        println!();
        contents.clear();
    }
}

fn eval(
    contents: &str,
    mem: &mut Vec<u8>,
    cellptr: &mut usize,
    debug: bool,
    getch: &Getch,
    mem_size: usize,
) -> Result<(), &'static str> {
    let mut codeptr = 0;
    let mut bracemap: HashMap<usize, usize> = HashMap::new();
    let mut temp = Vec::new();
    let mut debug_count = 0;

    for (pos, code) in contents.chars().enumerate() {
        if code == '[' {
            temp.push(pos)
        } else if code == ']' {
            let start = temp.pop().unwrap();
            bracemap.insert(start, pos);
            bracemap.insert(pos, start);
        }
    }

    while codeptr < contents.len() {
        let code = contents.chars().nth(codeptr).unwrap();
        match code {
            '>' => {
                *cellptr += 1;
                if *cellptr > mem_size {
                    return Err("Memory index out of bound");
                }
                if *cellptr == mem.len() {
                    mem.push(0)
                }
            }
            '<' => {
                if *cellptr == 0 {
                    return Err("Memory index out of bound");
                }
                *cellptr -= 1;
            }
            '+' => mem[*cellptr] += 1,
            '-' => mem[*cellptr] -= 1,
            '.' => print!("{}", (mem[*cellptr]) as char),
            ',' => mem[*cellptr] = getch.getch().unwrap(),
            '[' => {
                if mem[*cellptr] == 0 {
                    codeptr = *bracemap.get(&codeptr).unwrap()
                }
            }
            ']' => {
                if mem[*cellptr] != 0 {
                    codeptr = *bracemap.get(&codeptr).unwrap()
                }
            }
            '#' if debug => {
                debug_count += 1;

                println!(
                    "\ndebug flag {} : {} {} {}",
                    debug_count, mem[*cellptr] as char, mem[*cellptr], cellptr
                )
            }
            _ => {}
        }
        codeptr += 1;
    }
    Ok(())
}
