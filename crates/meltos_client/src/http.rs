use async_trait::async_trait;
use reqwest::{header, Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use meltos::room::RoomId;
use meltos::schema::room::Opened;
use meltos::schema::room::{Join, Joined, Open};
use meltos::user::UserId;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::push::Pushable;

use crate::config::SessionConfigs;
use crate::error;

#[derive(Debug, Clone)]
pub struct HttpClient {
    configs: SessionConfigs,
    client: Client,
    base_uri: String,
}

impl HttpClient {
    pub fn new(base_uri: impl Into<String>, configs: SessionConfigs) -> Self {
        Self {
            configs,
            client: Client::new(),
            base_uri: base_uri.into(),
        }
    }

    #[inline(always)]
    pub const fn configs(&self) -> &SessionConfigs {
        &self.configs
    }

    pub async fn join(
        base_uri: &str,
        room_id: RoomId,
        user_id: Option<UserId>,
    ) -> error::Result<(Self, Bundle)> {
        let client = Client::new();
        let response = client
            .post(format!("{base_uri}/room/{room_id}/join"))
            .json(&Join {
                user_id,
            })
            .send()
            .await?;
        let joined: Joined = response_to_json(response).await?;
        Ok((
            Self {
                configs: SessionConfigs {
                    session_id: joined.session_id.clone(),
                    user_id: joined.user_id.clone(),
                    room_id,
                },
                client,
                base_uri: base_uri.to_string(),
            },
            joined.bundle,
        ))
    }

    pub async fn open(
        base_uri: &str,
        bundle: Bundle,
        user_id: Option<UserId>,
    ) -> error::Result<Self> {
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
        Ok(Self {
            configs: SessionConfigs::from(opened),
            client,
            base_uri: base_uri.to_string(),
        })
    }

    #[inline]
    pub async fn fetch(&self) -> error::Result<Bundle> {
        self.get("tvn/fetch").await
    }

    async fn get<D>(&self, path: &str) -> error::Result<D>
    where
        D: DeserializeOwned,
    {
        let response = self
            .client
            .get(format!(
                "http://localhost:3000/room/{}/tvn/fetch",
                self.configs.room_id
            ))
            .header(header::CONTENT_TYPE, "application/json")
            .header(
                header::SET_COOKIE,
                format!("session_id={}", self.configs.session_id),
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
                format!("session_id={}", self.configs.session_id),
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
            .post(format!(
                "{}/room/{}/tvn/push",
                self.base_uri, self.configs.room_id
            ))
            .header(
                header::SET_COOKIE,
                format!("session_id={}", self.configs.session_id),
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
