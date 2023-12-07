use std::io;
use std::io::{stdout, Write};

use clap::Parser;

mod join;
mod thread;

fn main() {
    let _args = StartupCommands::parse();

    let io = io::stdin();
    loop {
        print!("[meltos]>");
        let _ = stdout().flush();

        let mut buffer = String::new();
        io.read_line(&mut buffer).expect("failed read input");
        let command: RunCommands = RunCommands::parse_from(buffer.split(' '));
        println!("{command:?}");
    }
}


#[derive(Debug, Parser)]
#[clap(name = "meltos", author, about, version)]
pub enum StartupCommands {
    Join(join::Join),

    Open,
}


#[derive(Parser, Debug)]
pub enum RunCommands {
    #[clap(subcommand)]
    Thread(thread::ThreadCommands),
}
