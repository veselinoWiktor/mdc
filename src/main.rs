mod compiler;
mod storage;
mod cli;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use structopt::{StructOpt};
use crate::cli::setting::Settings;
use compiler::assembly::codegen::gen;
use crate::compiler::assembly::instruction_fixup::fixup_program;
use crate::compiler::assembly::replace_pseudos::replace_pseudos;
use crate::compiler::emit::emit_assembly;
use crate::compiler::parser::parse_program;
use crate::compiler::tackygen::emit_tacky;
use crate::compiler::tokenizer::tokenize;
use crate::storage::ast::PrettyFormatter;

fn main() {
    let mut options = Settings::from_args();

    call_gcc_preprocessor(&mut options.file_path);

    let source_code = fs::read_to_string(&options.file_path).unwrap();
    println!("{}", source_code);

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


    let codegen_ast = gen(tacky_ast);
    println!("Codegen AST:\n{:?}", codegen_ast);

    let replace_pseudos_ast = replace_pseudos(codegen_ast);
    println!("Replace pseudos AST:\n{:?}", replace_pseudos_ast);

    let fixup_ast = fixup_program(replace_pseudos_ast.1, replace_pseudos_ast.0);
    println!("Replace pseudos AST:\n{:?}", fixup_ast);

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

fn call_gcc_preprocessor(file_path: &mut PathBuf) {
    let binding = file_path.clone();
    let original_file_path = match binding.to_str() {
        Some(file_path) => file_path,
        None => unreachable!()
    };

    file_path.set_extension("i");

    let preprocessor_command =
        Command::new("gcc")
            .arg("-E")
            .arg("-P")
            .arg(original_file_path)
            .arg("-o")
            .arg(file_path)
            .status();

    match preprocessor_command {
        Ok(status) => {}
        Err(error) => {}
    }
}


