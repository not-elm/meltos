use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use meltos_util::macros::Display;
use meltos_util::serde::AsBinary;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::error;

pub mod mock;


#[async_trait]
#[auto_delegate::delegate]
pub trait SessionIo: Send + Sync {
    async fn insert(&self, session_id: SessionId, session: RTCSessionDescription) -> error::Result;


    async fn read(&self, session_id: SessionId) -> error::Result<Option<RTCSessionDescription>>;
}


#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Display)]
pub struct SessionId(String);


impl From<&RTCSessionDescription> for SessionId {
    fn from(value: &RTCSessionDescription) -> Self {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(&value.as_binary().unwrap());
        Self(hasher.digest().to_string())
    }
}
