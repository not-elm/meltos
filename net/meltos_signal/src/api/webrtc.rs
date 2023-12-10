use meltos::session::SessionId;
use meltos_util::macros::Deref;
use meltos_util::sync::arc_mutex::ArcMutex;
use std::collections::HashMap;

pub mod host;
pub mod user;


pub type BroadcastSender = tokio::sync::broadcast::Sender<Vec<u8>>;
pub type BroadcastReceiver = tokio::sync::broadcast::Receiver<Vec<u8>>;


#[derive(Debug, Clone, Deref, Default)]
pub struct SocketChannels(ArcMutex<HashMap<SessionId, BroadcastSender>>);


impl SocketChannels {
    pub async fn broadcast(&self, session_id: &SessionId) -> Option<BroadcastSender> {
        self.0.lock().await.get(session_id).cloned()
    }
}

#[cfg(test)]
pub mod test_util {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;

    use crate::api::webrtc::host::OfferParam;

    pub fn init_request(offer_param: &OfferParam) -> http::Request<Body> {
        Request::builder()
            .method(http::Method::POST)
            .uri("/host/init")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(offer_param).unwrap()))
            .unwrap()
    }
}
