use meltos::command::client;
use meltos::command::client::ClientCommand;
use meltos::command::request::discussion::global::GlobalCmd;
use meltos::command::request::discussion::DiscussionCmd;
use meltos::discussion::io::DiscussionIo;
use meltos::discussion::structs::id::DiscussionId;
use meltos::discussion::structs::message::{MessageNo, MessageText};
use meltos::user::UserId;

use crate::error;

pub struct DiscussionCommandExecutor<'a, Global> {
    user_id: UserId,
    global_io: &'a Global,
}


impl<'a, Global> DiscussionCommandExecutor<'a, Global>
where
    Global: DiscussionIo,
{
    #[inline]
    pub const fn new(
        user_id: UserId,
        global_io: &'a Global,
    ) -> DiscussionCommandExecutor<'a, Global> {
        Self { user_id, global_io }
    }

    pub async fn execute(self, cmd: DiscussionCmd) -> error::Result<Option<ClientCommand>> {
        match cmd {
            DiscussionCmd::Global(global) => self.exe_global(global).await,
        }
    }

    async fn exe_global(self, global: GlobalCmd) -> error::Result<Option<ClientCommand>> {
        match global {
            GlobalCmd::Create => self.create_global_discussion().await,

            GlobalCmd::Speak {
                discussion_id,
                user_id,
                message,
            } => {
                self.speak_global_discussion(discussion_id, user_id, message)
                    .await
            }

            GlobalCmd::Reply {
                discussion_id,
                user_id,
                message_no,
                message,
            } => {
                self.reply(discussion_id, user_id, message_no, message)
                    .await
            }

            GlobalCmd::Close { discussion_id } => self.close(discussion_id).await,
        }
    }

    #[inline]
    async fn create_global_discussion(self) -> error::Result<Option<ClientCommand>> {
        let thread_meta = self.global_io.new_discussion(self.user_id).await?;
        let command = client::discussion::global::GlobalCmd::Created { meta: thread_meta };

        Ok(Some(ClientCommand::Discussion(
            client::discussion::DiscussionCmd::Global(command),
        )))
    }


    #[inline]
    async fn speak_global_discussion(
        self,
        discussion_id: DiscussionId,
        user_id: UserId,
        message_text: MessageText,
    ) -> error::Result<Option<ClientCommand>> {
        let message = self
            .global_io
            .speak(&discussion_id, user_id, message_text)
            .await?;
        Ok(Some(ClientCommand::Discussion(
            client::discussion::DiscussionCmd::Global(
                client::discussion::global::GlobalCmd::Spoke {
                    discussion_id,
                    message,
                },
            ),
        )))
    }


    #[inline]
    async fn reply(
        self,
        discussion_id: DiscussionId,
        user_id: UserId,
        message_no: MessageNo,
        message_text: MessageText,
    ) -> error::Result<Option<ClientCommand>> {
        let reply = self
            .global_io
            .reply(&discussion_id, user_id, message_no, message_text)
            .await?;

        Ok(Some(ClientCommand::Discussion(
            client::discussion::DiscussionCmd::Global(
                client::discussion::global::GlobalCmd::Replied {
                    discussion_id,
                    reply,
                },
            ),
        )))
    }


    #[inline]
    async fn close(self, discussion_id: DiscussionId) -> error::Result<Option<ClientCommand>> {
        self.global_io.close(&discussion_id).await?;
        Ok(Some(ClientCommand::Discussion(
            client::discussion::DiscussionCmd::Global(
                client::discussion::global::GlobalCmd::Closed { discussion_id },
            ),
        )))
    }
}
