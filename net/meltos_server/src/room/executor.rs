use meltos::command::client;
use meltos::command::client::ClientCommand;
use meltos::command::request::discussion::DiscussionCmd;
use meltos::command::request::discussion::global::GlobalCmd;
use meltos::command::request::RequestCmd;
use meltos::discussion::io::DiscussionIo;
use meltos::discussion::structs::id::DiscussionId;
use meltos::discussion::structs::message::MessageText;
use meltos::room::RoomId;
use meltos::user::UserId;

use crate::error;

pub(crate) struct ServerCommandExecutor<'a, Global> {
    _room_id: RoomId,
    from: UserId,
    thread_io: &'a Global,
}

impl<'a, Global> ServerCommandExecutor<'a, Global>
    where Global: DiscussionIo
{
    pub fn new(
        room_id: RoomId,
        user_id: UserId,
        io: &'a Global,
    ) -> ServerCommandExecutor<'a, Global> {
        Self {
            _room_id: room_id,
            from: user_id,
            thread_io: io,
        }
    }

    pub async fn execute(self, command: RequestCmd) -> error::Result<Option<ClientCommand>> {
        match command {
            RequestCmd::Discussion(thread) => self.execute_thread_command(thread).await
        }
    }

    async fn execute_thread_command(self, thread_command: DiscussionCmd) -> error::Result<Option<ClientCommand>> {
        match thread_command {
            DiscussionCmd::Global(global) => self.exe_global_thread(global).await
        }
    }

    async fn exe_global_thread(self, global: GlobalCmd) -> error::Result<Option<ClientCommand>> {
        match global {
            GlobalCmd::Create => self.create_global_discussion().await,
            GlobalCmd::Speak {
                discussion_id, user_id, message
            } => {}
            GlobalCmd::Reply { discussion_id, user_id, message_no, message } => {}
            GlobalCmd::Close { discussion_id } => {}
        }
    }

    #[inline]
    async fn create_global_discussion(self) -> error::Result<Option<ClientCommand>> {
        let thread_meta = self.thread_io.new_discussion(self.from).await?;
        let command = client::discussion::global::GlobalCmd::Created {
            meta: thread_meta
        };

        Ok(Some(ClientCommand::Discussion(client::discussion::DiscussionCmd::Global(command))))
    }


    #[inline]
    async fn speak_global_discussion(self, discussion_id: DiscussionId, user_id: UserId, message_text: MessageText) -> error::Result<Option<ClientCommand>> {
        self.thread_io.speak(&discussion_id, user_id, message_text).await?;
        Ok(Some(ClientCommand::))
    }
}
