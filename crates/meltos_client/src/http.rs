use reqwest::{Client, header};

use meltos::schema::request::room::Open;
use meltos::schema::response::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_tvn::io::bundle::Bundle;

use crate::error;

pub struct HttpClient {
    client: Client,
    base_uri: String,
}

impl HttpClient {
    pub fn new(base_uri: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_uri: base_uri.into(),
        }
    }


    pub async fn open_room(
        &self,
        user_id: Option<UserId>,
        bundle: Bundle,
    ) -> error::Result<Opened> {
        let response = self
            .client
            .post(format!("{}/room/open", self.base_uri))
            .json(&Open {
                user_id,
                bundle,
            })
            .send()
            .await?;

        Ok(response.json().await?)
    }



     pub async fn push(
        &self,
        session_id: SessionId,
        bundle: &Bundle,
    ) -> error::Result<()> {
        let response = self
            .client
            .post(format!("{}/room/tvn/push", self.base_uri))
            .header(header::SET_COOKIE, format!("session_id={session_id}"))
            .json(bundle)
            .send()
            .await?;

        Ok(())
    }
}
