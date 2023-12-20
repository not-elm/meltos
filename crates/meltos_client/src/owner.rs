use async_trait::async_trait;
use reqwest::Client;

use meltos::room::RoomId;
use meltos::schema::response::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::push::Pushable;
use meltos_tvn::operation::Operations;

use crate::http::HttpClient;

pub struct RoomOwner<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub room_id: RoomId,
    pub session_id: SessionId,
    pub user_id: UserId,
    client: HttpClient,
    operations: Operations<Fs, Io>,
}


impl<Fs, Io> RoomOwner<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Read + std::io::Write,
{
    pub async fn open(fs: Fs, user_id: Option<UserId>) -> crate::error::Result<Self> {
        let operations = Operations::new_main(fs);
        operations.init.execute()?;

        let client = HttpClient::new("http://localhost:3000");
        let opened = operations
            .push
            .execute(&mut OpenClient {
                http: &client,
                user_id,
            })
            .await?;
        println!("{opened:?}");
        Ok(Self {
            room_id: opened.room_id,
            session_id: opened.session_id,
            user_id: opened.user_id,
            client,
            operations,
        })
    }
}


struct OpenClient<'a> {
    user_id: Option<UserId>,
    http: &'a HttpClient,
}


#[async_trait]
impl<'a> Pushable<Opened> for OpenClient<'a> {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<Opened, Self::Error> {
        self.http.open_room(self.user_id.clone(), bundle).await
    }
}
