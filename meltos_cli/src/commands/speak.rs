use async_trait::async_trait;
use clap::Args;

use meltos_core::discussion::id::DiscussionId;
use meltos_core::discussion::message::MessageText;
use meltos_core::schema::discussion::global::Speak;
use meltos_client::discussion::DiscussionClient;

use crate::commands::{load_configs, CommandExecutable};

#[derive(Args, Debug, Clone)]
pub struct SpeakArgs {
    discussion_id: String,
    text: String,
}

#[async_trait(? Send)]
impl CommandExecutable for SpeakArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;

        let discussion = DiscussionClient::new(configs);
        let spoke = discussion
            .speak(&Speak {
                discussion_id: DiscussionId(self.discussion_id),
                text: MessageText(self.text),
            })
            .await?;
        println!("{spoke:?}");
        Ok(())
    }
}
