use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use meltos::session::SessionId;

use crate::api::webrtc::SocketChannels;
use crate::HttpResult;

#[derive(Debug, Deserialize, Serialize)]
pub struct Param {
    session_id: SessionId,
}


pub async fn join(
    ws: WebSocketUpgrade,
    Query(param): Query<Param>,
    State(channels): State<SocketChannels>,
) -> HttpResult<Response> {
    if let Some(_channel) = channels.broadcast(&param.session_id).await {
        Ok(ws.on_upgrade(move |_socket| async {}))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
