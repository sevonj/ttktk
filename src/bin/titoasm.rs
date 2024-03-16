//! TTKTK - TTK-91 ToolKit
//! Compiler executable
use std::{env, fs};
use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::PathBuf;
use libttktk::compiler::compile;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.reverse();

    // Skip first arg, which is program name
    let _ = args.pop();

    if args.is_empty() {
        println!("No arguments given.");
        print_help();
        return;
    }

    let input_path: String = args.pop().unwrap();
    let mut output_path: Option<String> = None;

    // Collect options
    loop {
        match args.pop() {
            None => break,
            Some(arg) => {
                match arg.as_str() {

                    // Output file
                    "-o" => {
                        match args.pop() {
                            None => {
                                print_err_no_arg(arg);
                                return;
                            }
                            Some(outfile) => {
                                match output_path {
                                    None => output_path = Some(outfile),
                                    Some(_) => {
                                        print_err_opt_redefine(arg);
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    // Help
                    "-h" => print_help(),

                    // Invalid
                    _ => {
                        println!("Err: Invalid option '{}'", arg);
                        return;
                    }
                }
            }
        }
    }

    // Open input file
    let source;
    match fs::read_to_string(&input_path) {
        Ok(contents) => source = contents,
        Err(e) => {
            print_err_inputfile(input_path, e);
            return;
        }
    }

    // Compile
    let output;
    match compile(source) {
        Ok(out) => output = out,
        Err(e) => {
            print_err_compiler(e);
            return;
        }
    }

    // Write output file
    if output_path.is_none() {
        let mut path = PathBuf::from(input_path);
        path.set_extension("b91");
        output_path = Some(path.into_os_string().into_string().unwrap());
    }
    let mut file = File::create(output_path.unwrap()).unwrap();
    let _ = write!(file, "{}", output);
    println!("Success!");
}

fn print_help() {
    println!("TTKTK Assembler]");
    println!("Usage: titoasm [file] [options]...");
    println!("Options:");
    println!("-h | --help       Help");
    println!("-o <file>         Specify output file. Default is same as input, with extension changed to .b91");
}

fn print_err_opt_redefine(opt: String) {
    println!("Err: Option '{}' is already defined!", opt);
}

fn print_err_no_arg(opt: String) {
    println!("Err: Not enough argument for '{}'", opt);
}

fn print_err_inputfile(file: String, e: Error) {
    println!("Err: Could not read input file {}: {}", file, e)
}

fn print_err_compiler(e: String) {
    println!("Err: Couldn't compile: {}", e)
}
