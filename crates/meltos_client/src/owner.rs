use reqwest::Client;

use meltos::room::RoomId;
use meltos::schema::response::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::Operations;

pub struct RoomOwner<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub room_id: RoomId,
    pub session_id: SessionId,
    pub user_id: UserId,
    client: Client,
    operations: Operations<Fs, Io>,
}


impl<Fs, Io> RoomOwner<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Read + std::io::Write,
{
    pub async fn open(fs: Fs) -> crate::error::Result<Self> {
        let operations = Operations::new_main(fs);
        operations.init.execute()?;
        let bundle = operations.bundle.create()?;
        let client = Client::new();
        let opened = http_open(&bundle, &client).await?;

        Ok(Self {
            room_id: opened.room_id,
            session_id: opened.session_id,
            user_id: opened.user_id,
            client,
            operations,
        })
    }
}

async fn http_open(
    bundle: &Bundle,
    client: &Client,
) -> crate::error::Result<Opened> {
    let response = client
        .post("http://localhost:3000/room/open".to_string())
        .json(&bundle)
        .send()
        .await?;
    let opened = response.json::<Opened>().await?;
    Ok(opened)
}
