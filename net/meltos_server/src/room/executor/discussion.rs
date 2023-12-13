use meltos::command::client::discussion::global::{Closed, Created, Replied, Spoke};
use meltos::command::request::discussion::global::{Reply, Speak};
use meltos::discussion::id::DiscussionId;
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


    #[inline]
    pub async fn reply(self, reply: Reply) -> error::Result<Replied> {
        let reply_message = self
            .global_io
            .reply(
                self.user_id,
                reply.message_id.clone(),
                reply.text,
            )
            .await?;

        Ok(Replied {
            reply_message_id: reply.message_id,
            reply: reply_message,
        })
    }


    #[inline]
    pub async fn close(self, discussion_id: DiscussionId) -> error::Result<Closed> {
        self.global_io
            .close(&discussion_id)
            .await
            .map_err(|_| error::Error::DiscussionNotExists(discussion_id.clone()))?;
        Ok(Closed { discussion_id })
    }
}
