use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use meltos_util::macros::Display;
use meltos_util::serde::AsBinary;

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Display)]
pub struct SessionId(String);


impl From<&RTCSessionDescription> for SessionId {
    fn from(value: &RTCSessionDescription) -> Self {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(&value.as_binary().unwrap());
        Self(hasher.digest().to_string())
    }
}
