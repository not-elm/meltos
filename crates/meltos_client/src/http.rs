use reqwest::Client;

use meltos::schema::request::room::Open;
use meltos::schema::response::room::Opened;
use meltos::user::UserId;
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
}
