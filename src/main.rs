mod compiler;
mod storage;

use std::fs;
use std::path::{PathBuf};
use structopt::{StructOpt};
use crate::compiler::tokenizer::tokenize;

fn main() {
    let options = Options::from_args();

    let mut source_code = fs::read_to_string(options.file_path).unwrap();

    // before tokenize should call gcc preprocessor

    match tokenize(source_code.as_str()) {
        Ok(tokens) => {
            println!("tokens: {:?}", tokens);
        }
        Err(err) => panic!("{:?}", err),
    }

    if options.lex {
        return
    }
    else {

    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "MDC", about = "My Dummy Compiler Driver")]
pub struct Options {
    #[structopt(
        short = "l",
        long = "lex",
        help = "Directs it to run the lexer, but stop before parsing"
    )]
    lex: bool,

    #[structopt(
        short = "p",
        long = "parse",
        help = "Directs it to run the lexer and parser, but stop before assembly generation"
    )]
    parse: bool,

    #[structopt(
        short = "c",
        long = "codegen",
        help = "Directs it to perform lexing, parsing, and assembly generation, but stop before code emission"
    )]
    codegen: bool,

    #[structopt(
        short = "S",
        long = "assembly",
        help = "Directs it to emit only an assembly file, but not assemble or link it"
    )]
    emit_assembly: bool,

    #[structopt(parse(from_os_str))]
    file_path: PathBuf,
}
