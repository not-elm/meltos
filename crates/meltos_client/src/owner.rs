use reqwest::{header, Client};

use meltos::command::client::room::Opened;
use meltos::room::RoomId;
use meltos::user::SessionId;
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::Operations;

pub struct RoomOwner<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub room_id: RoomId,
    client: Client,
    operations: Operations<Fs, Io>,
}


impl<Fs, Io> RoomOwner<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Read + std::io::Write,
{
    pub async fn open(fs: Fs, session_id: &SessionId) -> crate::error::Result<Self> {
        let operations = Operations::new_main(fs);
        operations.init.execute()?;
        let bundle = operations.bundle.create()?;
        let client = Client::new();
        let room_id = http_open(&bundle, &client, session_id).await?;

        Ok(Self {
            room_id,
            client,
            operations,
        })
    }
}

async fn http_open(
    bundle: &Bundle,
    client: &Client,
    session_id: &SessionId,
) -> crate::error::Result<RoomId> {
    let response = client
        .post("http://localhost:3000/room/open".to_string())
        .header(header::SET_COOKIE, format!("session_id={session_id}"))
        .json(&bundle)
        .send()
        .await?;
    let opened = response.json::<Opened>().await?;
    Ok(opened.room_id)
}
