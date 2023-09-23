use std::process;

use ariadne::{Color, Fmt};

pub fn simple(msg: impl ToString) -> ! {
    eprintln!("{} {}", "Error:".fg(Color::Red), msg.to_string());
    process::exit(1)
}
