use async_trait::async_trait;
use reqwest::Client;

use meltos::room::RoomId;
use meltos::schema::request::room::{Join, Joined};
use meltos::user::{SessionId, UserId};
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::FileSystem;

use crate::room::tvn::TvnClient;

pub mod discussion;
pub mod tvn;

#[async_trait]
pub trait RoomClientIo<E = crate::error::Error>: Sized {
    /// Join the room.
    async fn join(session_id: SessionId, room_id: RoomId) -> std::result::Result<Self, E>;
}


pub struct RoomUser<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub room_id: RoomId,
    pub session_id: SessionId,
    pub user_id: UserId,
    client: Client,
    tvn: TvnClient<Fs, Io>,
}


impl<Fs, Io> RoomUser<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub async fn join(room_id: RoomId,  user_id: Option<UserId>, fs: Fs) -> Result<Self, crate::error::Error> {
        let client = Client::new();
        let joined = http_join(&client, user_id, &room_id).await?;
        let tvn = TvnClient::new(room_id.clone(), joined.session_id.clone(), fs);
        tvn.init(&BranchName(joined.user_id.to_string()), joined.bundle)?;

        Ok(Self {
            client,
            tvn,
            room_id,
            session_id: joined.session_id,
            user_id: joined.user_id,
        })
    }
}


async fn http_join(client: &Client, user_id: Option<UserId>, room_id: &RoomId) -> crate::error::Result<Joined> {
    let response = client
        .post(format!("http://localhost:3000/room/{room_id}/join"))
        .json(&Join{
            user_id
        })
        .send()
      
        .await?;
    Ok(response.json().await?)
}