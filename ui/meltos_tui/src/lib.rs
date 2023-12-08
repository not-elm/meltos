use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use meltos_thread::io::global::mock::MockGlobalThreadIo;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Line, Style, Stylize};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::{Frame, Terminal};

use meltos_thread::io::ThreadIo;

use crate::global::GlobalThreadsUi;
use crate::state::UiState;

mod global;
mod state;
mod state_list;

#[derive(Default)]
pub struct App {
    ui_state: UiState,
    global: GlobalThreadsUi<MockGlobalThreadIo>,
}


impl App {
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        self.global.mock().await;

        loop {
            if event::poll(Duration::from_millis(60)).is_err() {
                continue;
            }

            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char(v) => {
                            if let Some(num) = v.to_digit(10) {
                                self.ui_state.step(num as usize);
                            }
                        }
                        KeyCode::Right => self.ui_state.next_tab(),
                        KeyCode::Left => self.ui_state.previous_tab(),
                        _ => {}
                    }
                }
            }
        }
    }


    fn render(&mut self, frame: &mut Frame) {
        let size = frame.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(size);

        let block = Block::default().on_black().cyan();
        frame.render_widget(block, size);

        frame.render_widget(self.tabs(), chunks[0]);
        self.global.render(frame, chunks[1]);
    }


    fn tabs(&self) -> Tabs {
        let titles: Vec<Line> = self
            .ui_state
            .titles
            .iter()
            .map(|title| Line::from(title.to_string()))
            .collect();

        Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.ui_state.index)
            .style(Style::default())
            .highlight_style(Style::default().bold().on_black())
    }
}
