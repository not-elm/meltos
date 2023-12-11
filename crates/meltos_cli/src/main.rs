use std::error::Error;
use std::io;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::prelude::*;

use meltos_tui::App;

mod join;
mod thread;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::default();
    let res = app.run(&mut terminal).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
    // let _args = StartupCommands::parse();
    //
    // let io = io::stdin();
    // loop {
    //     print!("[meltos]>");
    //     let _ = stdout().flush();
    //
    //     let mut buffer = String::new();
    //     io.read_line(&mut buffer).expect("failed read input");
    //     let command: RunCommands = RunCommands::parse_from(buffer.split('
    // '));     println!("{command:?}");
    // }
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
