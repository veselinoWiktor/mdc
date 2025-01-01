mod compiler;
mod storage;
mod cli;

use std::fs;
use std::process::Command;
use structopt::{StructOpt};
use crate::cli::setting::Settings;
use crate::compiler::codegen::gen;
use crate::compiler::emit::emit_assembly;
use crate::compiler::parser::parse_program;
use crate::compiler::tackygen::emit_tacky;
use crate::compiler::tokenizer::tokenize;
use crate::storage::ast::PrettyFormatter;

fn main() {

    let mut options = Settings::from_args();

    let binding = options.file_path.clone();
    let original_file_path = binding.to_str().unwrap();
    options.file_path.set_extension("i");

    let preprocessor_command =
        Command::new("gcc")
            .arg("-E")
            .arg("-P")
            .arg(original_file_path)
            .arg("-o")
            .arg(&options.file_path)
            .status();

    match preprocessor_command {
        Err(_) => panic!("File was unable to preprocess"),
        _ => ()
    }

    let source_code = fs::read_to_string(&options.file_path).unwrap();

    println!("{}", source_code);

    // before tokenize should call gcc preprocessor
    let mut tokens = match tokenize(source_code.as_str()) {
        Ok(tokens) => {
            println!("Tokens {:?}", tokens);
            tokens
        }
        Err(err) => panic!("{:?}", err),
    };

    if options.lex {
        return
    }

    let ast = match parse_program(&mut tokens) {
        Ok(ast) => {
            println!("AST:\n{:?}", ast);
            ast
        }
        Err(err) => panic!("{:?}", err)
    };

    if options.parse
    {
        return;
    }

    let tacky_ast = emit_tacky(ast);
    println!("TACKY AST:\n{:?}", tacky_ast);

    if options.tacky
    {
        return;
    }

    //let assembly_ast = gen(ast);
    //println!("Assembly AST:\n{:?}", assembly_ast);

    if options.codegen
    {
        return;
    }

    // let assembly_source_code = emit_assembly(assembly_ast);
    // options.file_path.set_extension("s");
    // fs::write(&options.file_path, &assembly_source_code).unwrap();

    if options.emit_assembly
    {
        return;
    }
}


