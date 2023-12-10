use axum::extract::State;
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::session::{SessionId, SessionIo};
use crate::state::SessionIoState;
use crate::HttpResult;

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct OfferParam {
    pub session_description: RTCSessionDescription,
}


pub async fn init<S>(
    State(session_io): State<SessionIoState<S>>,
    Json(param): Json<OfferParam>,
) -> HttpResult<String>
where
    S: SessionIo + Clone,
{
    let session_id = SessionId::from(&param.session_description);
    let session_id_str = session_id.to_string();
    match session_io
        .insert(session_id, param.session_description)
        .await
    {
        Ok(()) => Ok(session_id_str),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}
