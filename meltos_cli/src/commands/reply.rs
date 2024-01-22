use async_trait::async_trait;
use clap::Args;

use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{MessageId, MessageText};
use meltos::schema::discussion::global::Reply;
use meltos_client::discussion::DiscussionClient;

use crate::commands::{load_configs, CommandExecutable};

#[derive(Args, Debug, Clone)]
pub struct ReplyArgs {
    discussion_id: String,
    to_message_id: String,
    text: String,
}

#[async_trait(? Send)]
impl CommandExecutable for ReplyArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let discussion = DiscussionClient::new(configs);
        let replied = discussion
            .reply(&Reply {
                discussion_id: DiscussionId(self.discussion_id),
                to: MessageId(self.to_message_id),
                text: MessageText(self.text),
            })
            .await?;
        println!("{replied:?}");
        Ok(())
    }
}
