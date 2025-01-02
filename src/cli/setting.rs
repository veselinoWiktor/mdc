use std::path::PathBuf;
use structopt_derive::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "MDC", about = "My Dummy Compiler Driver")]
pub struct Settings {
    #[structopt(parse(from_os_str))]
    pub file_path: PathBuf,

    #[structopt(
        short = "l",
        long = "lex",
        help = "Directs it to run the tokenizer, but stop before parsing"
    )]
    pub lex: bool,

    #[structopt(
        short = "p",
        long = "parse",
        help = "Directs it to run the tokenizer and parser, but stop before assembly generation"
    )]
    pub parse: bool,

    #[structopt(
        short = "t",
        long = "tacky",
        help = "Directs it to run the tokenizer, parser and tacky generation, but stop before assembly generation"
    )]
    pub tacky: bool,

    #[structopt(
        short = "c",
        long = "codegen",
        help = "Directs it to perform tokenizer, parsing, tacky generation and assembly generation, but stop before code emission"
    )]
    pub codegen: bool,

    #[structopt(
        short = "S",
        long = "assembly",
        help = "Directs it to emit only an assembly file, but not assemble or link it"
    )]
    pub emit_assembly: bool,
}