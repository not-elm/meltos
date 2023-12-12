use meltos::command::client::discussion::global::{Created, Spoke};
use meltos::command::request::discussion::global::Speak;
use meltos::user::UserId;
use meltos_backend::discussion::DiscussionIo;

use crate::error;

pub struct DiscussionCommandExecutor<'a, Global: ?Sized> {
    user_id: UserId,
    global_io: &'a Global,
}


impl<'a, Global> DiscussionCommandExecutor<'a, Global>
    where
        Global: DiscussionIo + ?Sized,
{
    #[inline]
    pub const fn new(
        user_id: UserId,
        global_io: &'a Global,
    ) -> DiscussionCommandExecutor<'a, Global> {
        Self { user_id, global_io }
    }


    #[inline]
    pub async fn create(self) -> error::Result<Created> {
        let discussion_meta = self.global_io.new_discussion(self.user_id).await?;
        Ok(Created {
            meta: discussion_meta,
        })
    }


    #[inline]
    pub async fn speak(self, speak: Speak) -> error::Result<Spoke> {
        let message = self
            .global_io
            .speak(&speak.discussion_id, self.user_id, speak.message)
            .await?;
        Ok(Spoke {
            discussion_id: speak.discussion_id,
            message,
        })
    }
    //
    //
    // #[inline]
    // async fn reply(
    //     self,
    //     discussion_id: DiscussionId,
    //     user_id: UserId,
    //     message_no: MessageNo,
    //     message_text: MessageText,
    // ) -> error::Result<Option<ClientCommand>> {
    //     let reply = self
    //         .global_io
    //         .reply(&discussion_id, user_id, message_no, message_text)
    //         .await?;
    //
    //     Ok(Some(ClientCommand::Discussion(
    //         client::discussion::DiscussionCmd::Global(
    //             client::discussion::global::GlobalCmd::Replied {
    //                 discussion_id,
    //                 reply,
    //             },
    //         ),
    //     )))
    // }
    //
    //
    // #[inline]
    // async fn close(self, discussion_id: DiscussionId) -> error::Result<Option<ClientCommand>> {
    //     self.global_io.close(&discussion_id).await?;
    //     Ok(Some(ClientCommand::Discussion(
    //         client::discussion::DiscussionCmd::Global(
    //             client::discussion::global::GlobalCmd::Closed { discussion_id },
    //         ),
    //     )))
    // }
}
