use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct OfferParam {
    pub session_description: RTCSessionDescription,
}
