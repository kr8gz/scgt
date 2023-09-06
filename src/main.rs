use std::{env, fs};

mod error;
mod parser;
mod helper_functions;

fn main() {
    let mut args = env::args();
    match args.nth(1) {
        None => error::simple("The file to process is a required argument."),
        Some(file) => {
            let code = fs::read_to_string(file)
                .unwrap_or_else(|err| error::simple(err))
                .replace("\r\n", "\n");

            let result = parser::parse(&code);
            let (output, errors) = result.into_output_errors();
            dbg!(errors);
            if let Some(output) = output {
                fs::write("output.spwn", output).unwrap_or_else(|err| error::simple(err));
            }
        }
    }
}
