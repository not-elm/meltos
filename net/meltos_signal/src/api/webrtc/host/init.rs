use axum::extract::State;
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use meltos::session::SessionId;

use crate::session::SessionIo;
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


#[cfg(test)]
mod tests {
    use axum::http;
    use http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use meltos::session::SessionId;

    use crate::api::webrtc::host::init::OfferParam;
    use crate::api::webrtc::test_util::init_request;
    use crate::app;
    use crate::session::mock::MockSessionIo;

    #[tokio::test]
    async fn offer() {
        let app = app::<MockSessionIo>();
        let offer = OfferParam::default();
        let response = app.oneshot(init_request(&offer)).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body,
            SessionId::from(&offer.session_description)
                .to_string()
                .as_bytes()
        );
    }
}
