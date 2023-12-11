use meltos::command::client::discussion::global::Created;
use meltos::discussion::io::DiscussionIo;
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


    #[inline]
    pub async fn create(self) -> error::Result<Created> {
        let discussion_meta = self.global_io.new_discussion(self.user_id).await?;
        Ok(Created {
            meta: discussion_meta,
        })
    }

    //
    // #[inline]
    // async fn speak_global_discussion(
    //     self,
    //     discussion_id: DiscussionId,
    //     user_id: UserId,
    //     message_text: MessageText,
    // ) -> error::Result<Option<ClientCommand>> {
    //     let message = self
    //         .global_io
    //         .speak(&discussion_id, user_id, message_text)
    //         .await?;
    //     Ok(Some(ClientCommand::Discussion(
    //         client::discussion::DiscussionCmd::Global(
    //             client::discussion::global::GlobalCmd::Spoke {
    //                 discussion_id,
    //                 message,
    //             },
    //         ),
    //     )))
    // }
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
