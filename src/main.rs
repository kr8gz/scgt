use std::fs;

use clap::Parser;

mod lex;
mod parse;
mod util;

#[derive(Parser, Debug)]
#[command(author = "kr8gz", verbatim_doc_comment)]
/// A golfing language that compiles to SPWN code.
struct Args {
    /// The path to the SCGT file to be compiled.
    // #[arg(forbid_empty_values = true)]
    file: String,

    #[arg(short, long, value_name = "SPACES", default_value_t = 4)]
    /// Indentation size for generated SPWN code.
    indent_size: usize,
}

fn main() {
    // TODO uncomment when finish RWRT

    // let args = Args::parse();
    // let code = fs::read_to_string(args.file)
    //     .unwrap_or_else(|err| util::errors::simple(err))
    //     .replace("\r\n", "\n");

    // let result = parser::parse(&code, args.indent_size);
    // let (output, errors) = result.into_output_errors();
    // dbg!(errors);
    // if let Some(output) = output {
    //     fs::write("output.spwn", output).unwrap_or_else(|err| util::errors::simple(err));
    // }
}
