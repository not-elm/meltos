use meltos::thread::io::ThreadIo;
use meltos::thread::structs::message::{Message, MessageText, Messages};
use meltos::thread::structs::MessageThread;
use meltos::user::UserId;
use ratatui::widgets::List;
use ratatui::{prelude::*, widgets::*};


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
    list: StatefulList<MessageThread>,
    threads: Threads,
}


impl<Threads> GlobalThreadsUi<Threads>
where
    Threads: ThreadIo,
{
    pub async fn mock(&mut self) {
        let id = self.threads.new_thread().await.unwrap();
        self.threads
            .speak(&id, UserId::from("user"), MessageText::from("hello"))
            .await
            .unwrap();
        self.list.items = self.threads.thread_all().await.unwrap();
    }

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
                ListItem::new(Line::from(thread.id.to_string()))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
            })
            .collect()
    }
}
