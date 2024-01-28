use meltos::discussion::id::DiscussionId;
use meltos::schema::discussion::global::{Closed, Created, Replied, Spoke};
use meltos::schema::discussion::global::{Reply, Speak};
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
        Self {
            user_id,
            global_io,
        }
    }

    #[inline]
    pub async fn create(self, title: String) -> error::Result<Created> {
        let discussion_meta = self
            .global_io
            .new_discussion(title, self.user_id)
            .await?;
        Ok(Created {
            meta: discussion_meta,
        })
    }

    #[inline]
    pub async fn speak(self, speak: Speak) -> error::Result<Spoke> {
        let message = self
            .global_io
            .speak(&speak.discussion_id, self.user_id, speak.text)
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
            .reply(reply.discussion_id.clone(), self.user_id, reply.to.clone(), reply.text)
            .await?;

        Ok(Replied {
            discussion_id: reply.discussion_id,
            to: reply.to,
            message: reply_message,
        })
    }

    #[inline]
    pub async fn close(self, discussion_id: DiscussionId) -> error::Result<Closed> {
        self.global_io
            .close_discussion(&discussion_id)
            .await?;
        Ok(Closed {
            discussion_id,
        })
    }
}
