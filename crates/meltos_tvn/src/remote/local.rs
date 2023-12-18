use async_trait::async_trait;

use meltos::room::RoomId;

use crate::operation::push::PushParam;
use crate::remote::CommitPushable;

pub struct LocalHttpClient {
    client: reqwest::Client,
    room_id: RoomId,
}


impl LocalHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            room_id: RoomId("".to_string()),
        }
    }
}


#[async_trait]
impl CommitPushable for LocalHttpClient {
    type Error = reqwest::Error;

    async fn push(&mut self, param: PushParam) -> std::result::Result<(), Self::Error> {
        self.client
            .post(format!(
                "http:localhost:3000/room/{}/tvn/push",
                self.room_id
            ))
            .json(&param)
            .send()
            .await?;

        Ok(())
    }
}
