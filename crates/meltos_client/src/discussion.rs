use async_trait::async_trait;

use meltos_core::discussion::id::DiscussionId;
use meltos_core::discussion::message::Message;
use meltos_core::discussion::DiscussionMeta;
use meltos_core::schema::discussion::global::{Replied, Reply, Speak, Spoke};

use crate::config::SessionConfigs;
use crate::error;
use crate::http::HttpClient;
use crate::tvc::BASE;

#[async_trait]
pub trait ClientDiscussionIo: Send + Sync {
    type Error: std::error::Error;

    async fn created(&self, discussion_meta: DiscussionMeta) -> Result<(), Self::Error>;

    async fn spoke(
        &self,
        discussion_id: DiscussionId,
        messages: Message,
    ) -> Result<(), Self::Error>;

    async fn replied(&self, discussion_id: DiscussionId, reply: Message)
        -> Result<(), Self::Error>;

    async fn closed(&self, discussion_id: DiscussionId) -> Result<(), Self::Error>;
}

pub struct DiscussionClient {
    http: HttpClient,
}

impl DiscussionClient {
    #[inline(always)]
    pub fn new(config: SessionConfigs) -> Self {
        Self {
            http: HttpClient::new(BASE, config),
        }
    }

    #[inline(always)]
    pub async fn speak(&self, speak: &Speak) -> error::Result<Spoke> {
        self.http.speak(speak).await
    }

    #[inline(always)]
    pub async fn reply(&self, reply: &Reply) -> error::Result<Replied> {
        self.http.reply(reply).await
    }
}
