use ratatui::widgets::List;
use ratatui::{prelude::*, widgets::*};

use meltos::discussion::io::DiscussionIo;
use meltos::discussion::structs::message::{Message, Messages};
use meltos::discussion::structs::Discussion;

use crate::state_list::StatefulList;

#[derive(Default)]
pub enum GlobalViewState {
    #[default]
    SideBar,

    Content,
}

#[derive(Default)]
pub struct GlobalThreadsUi<Threads> {
    state: GlobalViewState,
    list: StatefulList<Discussion>,
    threads: Threads,
}


impl<Threads> GlobalThreadsUi<Threads>
where
    Threads: DiscussionIo,
{
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(50)])
            .split(area);

        frame.render_stateful_widget(self.sidebar(), chunks[0], &mut self.list.state);
        frame.render_widget(self.thread_view(&self.list.items[0].messages), chunks[1]);
    }


    fn thread_view<'a>(&self, messages: &Messages) -> List<'a> {
        List::new(
            messages
                .iter()
                .map(|m| self.message(m))
                .collect::<Vec<ListItem<'a>>>(),
        )
        .block(Block::new().borders(Borders::ALL).title("Threads"))
        .style(Style::default())
        .highlight_style(Style::default().bold().on_black())
    }


    fn message<'a>(&self, message: &Message) -> ListItem<'a> {
        ListItem::new(Line::from(format!(
            "{} {} >> {:?}",
            message.no,
            &message.user_id,
            message.text.to_string()
        )))
    }


    fn sidebar<'a>(&self) -> List<'a> {
        List::new(self.sidebar_items())
            .block(
                Block::default()
                    .on_black()
                    .borders(Borders::ALL)
                    .title("Threads"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
    }

    fn sidebar_items<'a>(&self) -> Vec<ListItem<'a>> {
        self.list
            .items
            .iter()
            .map(|thread| {
                ListItem::new(Line::from(thread.meta.id.to_string()))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
            })
            .collect()
    }
}
