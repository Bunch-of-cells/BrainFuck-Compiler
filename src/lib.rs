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
    verbose: bool,
    run: bool,
    output: &'a str,
    keep: bool,
    compiler: &'a str,
}

struct ArgFlags(u16);

impl ArgFlags {
    const HELP: u16 = 1;
    const FILE: u16 = 2;
    const OUTPUT: u16 = 4; //I
    const KEEP: u16 = 8; //I
    const COMPILER: u16 = 16; //I
    const RUN: u16 = 32; //I
    const INTERPRET: u16 = 64;
    const DEBUG: u16 = 128;
    const MEM_SIZE: u16 = 256;
    const OFFSET: u16 = 512;
    const RELEASE: u16 = 1024; //I
    const VERBOSE: u16 = 2048;
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
            verbose: false,
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
                println!("  --debug | -d           Activates the debug mode.\n\t\t\t In the debug mode, any # or | will be considered as a debug symbol");
                println!("  --mem_size | -m        Set the memory, default is 30000");
                println!("  --release | -rl        Compiles in release mode");
                println!("  --verbose | -v         Compiles VerboseFuck");
                println!("  --ptr-offset | -po     Set the pointer offset from the start of the memory, default is 0\n");
            }
            "--keep" | "-k" => {
                if flags.0 & ArgFlags::KEEP != 0 {
                    return Err("More than 1 build flag passed".to_owned());
                }
                flags.0 |= ArgFlags::KEEP;
                parsed_args.keep = true;
            }
            "--verbose" | "-v" => {
                if flags.0 & ArgFlags::VERBOSE != 0 {
                    return Err("Compile in VerboseFuck".to_owned());
                }
                flags.0 |= ArgFlags::VERBOSE;
                parsed_args.verbose = true;
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
                        return Err(format!(
                            "Invalid file name: '{}'. File must end with .bf",
                            other
                        ));
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
    validate_args(flags, parsed_args, args.len())
}

fn validate_args(flags: ArgFlags, mut args: Args, len: usize) -> Result<Args, String> {
    if args.offset > args.mem_size {
        return Err("pointer offset cannot be greater than memory size".to_owned());
    }
    if flags.0 & 1084 != 0 && flags.0 & ArgFlags::INTERPRET != 0 {
        return Err("Invalid argument combination".to_owned());
    }
    if (flags.0 & 62591 == 0 && len <= 3) || flags.0 == 0 {
        args.console = true;
        return Ok(args);
    }
    if flags.0 & 64387 != 0 && flags.0 & ArgFlags::FILE == 0 {
        return Err("No File passed".to_owned());
    }
    Ok(args)
}

pub fn get_code(filename: &str) -> Result<String, String> {
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => return Err(err.to_string()),
    };

    if contents.matches('[').count() != contents.matches(']').count() {
        return Err("Unbalanced Brackets".to_string());
    }
    Ok(contents)
}

pub fn interpret(mut contents: String, args: Args) -> Result<(), String> {
    let mut mem = vec![0];
    for _ in 0..args.offset {
        mem.push(0)
    }
    if !args.verbose {
        contents.retain(|c| "<>[]+-.,#|".contains(c))
    }
    let mut cellptr = args.offset;
    let mut debug_count = 0;
    let getch = Getch::new().unwrap();
    let mut codeptr = 0;
    let mut bracemap: HashMap<usize, usize> = HashMap::new();
    let mut temp = Vec::new();
    let mut error = None;

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
                if cellptr > args.mem_size {
                    return Err("Memory index out of bound".to_owned());
                }
                if cellptr == mem.len() {
                    mem.push(0)
                }
            }
            '<' => {
                if cellptr == 0 {
                    return Err("Memory index out of bound".to_owned());
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
            '#' if args.debug => {
                debug_count += 1;

                println!(
                    "\ndebug flag {} : {} {} {}",
                    debug_count, mem[cellptr] as char, mem[cellptr], cellptr
                )
            }
            '|' if args.debug => {
                println!(
                    "\n{:?}",
                    &mem[(cellptr as i32 - 10).clamp(0, args.mem_size as i32) as usize
                        ..=(cellptr + 10).clamp(0, args.mem_size)]
                )
            }
            ch => {
                error = Some(format!("Invalid BrainFuck character: '{}'", ch));
                break;
            }
        }
        codeptr += 1;
    }
    println!("\n\x1b[90m----------------------------------\x1b[0m");
    if let Some(err) = error {
        return Err(err);
    }
    Ok(())
}

fn translate(contents: &str, debug: bool, mem: usize, offset: usize) -> Result<String, String> {
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
    unsigned char mem[{}];
    unsigned char* ptr = mem + {};

",
        mem, offset
    );
    if debug {
        cpp_code.push_str("\tunsigned int debug_count = 0;\n")
    }
    let gen_code = gen_optimized(contents.to_string(), debug, mem)?;
    cpp_code += &gen_code;
    cpp_code.push_str("\treturn 0;\n}\n");
    Ok(cpp_code)
}

pub fn compile(mut contents: String, args: Args) -> Result<(), Box<dyn Error>> {
    if !args.verbose {
        contents.retain(|c| "<>[]+-.,#|".contains(c));
    }
    println!("\x1b[1mCreating the C file...\x1b[0m");
    let mut cpp_file = File::create([args.output, ".c"].concat())?;
    cpp_file.write_all(
        translate(
            &contents,
            args.debug && !args.release,
            args.mem_size,
            args.offset,
        )?
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

pub fn run_in_terminal(args: Args) -> Result<(), String> {
    let getch = Getch::new().unwrap();
    let mut mem = vec![0];
    for _ in 0..args.offset {
        mem.push(0)
    }
    let mut cellptr = args.offset;
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
        if !args.verbose {
            contents.retain(|c| "<>[]+-.,#|".contains(c));
        }
        eval(
            &contents,
            &mut mem,
            &mut cellptr,
            args.debug,
            &getch,
            args.mem_size,
        )?;
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
) -> Result<(), String> {
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
                    return Err("Memory index out of bound".to_owned());
                }
                if *cellptr == mem.len() {
                    mem.push(0)
                }
            }
            '<' => {
                if *cellptr == 0 {
                    return Err("Memory index out of bound".to_owned());
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
            '|' if debug => {
                println!(
                    "\n{:?}",
                    &mem[(*cellptr as i32 - 10).clamp(0, mem.len() as i32) as usize
                        ..=(*cellptr + 10).clamp(0, mem.len())]
                )
            }
            ch => return Err(format!("Invalid BrainFuck character: '{}'", ch)),
        }
        codeptr += 1;
    }
    Ok(())
}

pub fn verbosify(filename: &str) -> Result<(), String> {
    println!("\x1b[1mOpening {}...\x1b[0m", filename);
    let mut contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => return Err(err.to_string()),
    };
    println!("\x1b[1mVerbosifying...\x1b[0m");
    contents.retain(|c| "<>[]+-.,".contains(c));
    let mut cpp_file = match File::create(filename) {
        Ok(x) => x,
        Err(err) => return Err(err.to_string()),
    };
    println!("\x1b[1mSaving {}...\x1b[0m", filename);
    match cpp_file.write_all(contents.as_bytes()) {
        Ok(()) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

pub fn gen_optimized(mut code: String, debug: bool, mem_len: usize) -> Result<String, String> {
    let mut gen_code = String::new();
    while code.contains("><")
        || code.contains("<>")
        || code.contains("+-")
        || code.contains("-+")
        || code.contains("[-]")
        || code.contains("[+]")
    {
        code = code.replace("><", "");
        code = code.replace("<>", "");
        code = code.replace("+-", "");
        code = code.replace("-+", "");
        code = code.replace("[-]", "c");
        code = code.replace("[+]", "c");
    }
    let mut chars = code.chars().peekable();
    while let Some(op) = chars.next() {
        gen_code.push_str(&match op {
            '>' | '<' => {
                let mut counter = if op == '>' { 1 } else { -1 };
                loop {
                    counter += match chars.peek() {
                        Some('>') => 1,
                        Some('<') => -1,
                        _ => break
                    };
                    chars.next();
                }
                if counter == 0 {"".to_owned()}
                else {format!("\tptr += {};\n", counter)}
            },
            '+' | '-' | ',' | 'c' => {
                let mut counter = if op == '+' { 1 } else if op == '-' { -1 } else {0};
                let mut getch = (op == ',') as usize;
                let mut c = op == 'c';
                loop {
                    counter += match chars.peek() {
                        Some('+') => 1,
                        Some('-') => -1,
                        Some('c') => {
                            c = true;
                            -counter
                        },
                        Some(',') => {
                            getch += 1;
                            -counter
                        }
                        _ => break
                    };
                    chars.next();
                }
                if getch > 0 {
                    format!("\t{}\n\t*ptr = getch() + {};\n", "getch();".repeat(getch - 1), counter)
                }
                else if c {format!("\t*ptr = {};\n", counter)}
                else if counter == 0 {"".to_owned()}
                else {format!("\t*ptr += {};\n", counter)}
            },
            '.' => "\tputchar(*ptr);\n".to_owned(),
            '[' => "\twhile (*ptr) {\n".to_owned(),
            ']' => "\t}\n".to_owned(),
            '#' if debug => {
                "\tdebug_count += 1;printf(\"\\ndebug flag %d : %c, %d, %ld\\n\", debug_count, *ptr, *ptr, ptr-mem);\n".to_owned()
            }
            '|' if debug => {
                format!("\tprintf(\"\\n\");for (unsigned char* i = ptr - 15; i < ptr + 15; i++) {{if (i < mem || i > mem+{}) continue; if (i == ptr) printf(\"|%d| \", *i); else printf(\"%d \", *i); }}printf(\"\\n\");\n", mem_len)
            }
            _ => {return Err(format!("Invalid BrainFuck character: '{}'", op))},
        });
    }
    Ok(gen_code)
}
