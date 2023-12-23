use async_trait::async_trait;
use meltos::room::RoomId;
use reqwest::{header, Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use meltos::schema::request::room::{Join, Joined, Open};
use meltos::schema::response::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::push::Pushable;

use crate::config::SessionConfigs;
use crate::error;

#[derive(Debug, Clone)]
pub struct HttpClient {
    session_id: SessionId,
    client: Client,
    base_uri: String,
}


impl HttpClient {
    pub fn new(base_uri: impl Into<String>, session_id: SessionId) -> Self {
        Self {
            session_id,
            client: Client::new(),
            base_uri: base_uri.into(),
        }
    }


    pub async fn join(
        base_uri: &str,
        room_id: &RoomId,
        user_id: Option<UserId>,
    ) -> error::Result<(Self, Joined)> {
        let client = Client::new();
        let response = client
            .post(format!("{base_uri}/room/{room_id}/join"))
            .json(&Join {
                user_id,
            })
            .send()
            .await?;
        let joined: Joined = response_to_json(response).await?;
        let me = Self {
            session_id: joined.session_id.clone(),
            client,
            base_uri: base_uri.to_string(),
        };

        Ok((me, joined))
    }


    pub async fn open(
        base_uri: &str,
        bundle: Bundle,
        user_id: Option<UserId>,
    ) -> error::Result<(Self, SessionConfigs)> {
        let client = Client::new();
        let response = client
            .post(format!("{base_uri}/room/open"))
            .json(&Open {
                user_id,
                bundle,
            })
            .send()
            .await?;
        let opened: Opened = response.error_for_status()?.json().await?;
        let configs = SessionConfigs::from(opened);
        let me = Self {
            session_id: configs.session_id.clone(),
            client,
            base_uri: base_uri.to_string(),
        };
        Ok((me, configs))
    }


    #[inline]
    pub async fn fetch(&self) -> error::Result<Bundle> {
        self.get("/room/tvn/fetch").await
    }


    async fn get<D>(&self, path: &str) -> error::Result<D>
    where
        D: DeserializeOwned,
    {
        let response = self
            .client
            .get(format!("{}/{path}", self.base_uri))
            .header(
                header::SET_COOKIE,
                format!("session_id={}", self.session_id),
            )
            .send()
            .await?;
        response_to_json(response).await
    }

    async fn post<S, D>(&self, path: &str, body: &S) -> error::Result<D>
    where
        S: Serialize,
        D: DeserializeOwned,
    {
        let response = self
            .client
            .post(format!("{}/{path}", self.base_uri))
            .header(
                header::SET_COOKIE,
                format!("session_id={}", self.session_id),
            )
            .json(body)
            .send()
            .await?;
        response_to_json(response).await
    }
}


#[async_trait]
impl Pushable<()> for HttpClient {
    type Error = error::Error;

    async fn push(&mut self, bundle: Bundle) -> error::Result<()> {
        let response = self
            .client
            .post(format!("{}/room/tvn/push", self.base_uri))
            .header(
                header::SET_COOKIE,
                format!("session_id={}", self.session_id),
            )
            .json(&bundle)
            .send()
            .await?;
        response.error_for_status()?;
        Ok(())
    }
}

async fn response_to_json<D>(response: Response) -> error::Result<D>
where
    D: DeserializeOwned,
{
    Ok(response.error_for_status()?.json().await?)
}
