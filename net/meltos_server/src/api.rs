use axum::body::Body;
use axum::response::Response;
use serde::Serialize;

use meltos_util::serde::SerializeJson;

pub mod login;
pub mod room;


type HttpResult = std::result::Result<Response, Response>;


pub trait AsSuccessResponse {
    fn as_success_response(&self) -> Response;
}


impl<D> AsSuccessResponse for D
    where
        D: Serialize,
{
    fn as_success_response(&self) -> Response {
        Response::builder()
            .body(Body::from(self.as_json()))
            .unwrap()
    }
}


#[cfg(test)]
mod test_util {
    use axum::{async_trait, http, Router};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::header;
    use axum::response::Response;
    use http_body_util::BodyExt;
    use tower::{Service, ServiceExt};

    use meltos::command::client::discussion::global::Created;
    use meltos::command::client::room::Opened;
    use meltos::room::RoomId;
    use meltos::user::{SessionId, UserId};
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;

    use crate::app;


    #[async_trait]

    pub trait ResponseConvertable{
        async fn into_json(self) -> String;
    }


    #[async_trait]

    impl ResponseConvertable for Response{
        async fn into_json(self) -> String {
            let bytes = self.into_body().collect().await.unwrap()
                .to_bytes()
                .to_vec();
            String::from_utf8(bytes).unwrap()
        }
    }


    pub async fn logged_in_app() -> (SessionId, Router) {
        let session = MockUserSessionIo::default();
        session
            .register(mock_session_id(), UserId::from("user"))
            .await
            .unwrap();
        (mock_session_id(), app(session, MockGlobalDiscussionIo::default()))
    }


    pub async fn open_room(app: &mut Router, user_token: SessionId) -> RoomId {
        let response = ServiceExt::<Request<Body>>::ready(app)
            .await
            .unwrap()
            .call(open_room_request(user_token))
            .await
            .unwrap();
        let response = serde_json::from_slice::<Opened>(
            &response.into_body().collect().await.unwrap().to_bytes(),
        )
            .unwrap();
        response.room_id
    }

    pub fn mock_session_id() -> SessionId {
        SessionId("session_id".to_string())
    }


    pub fn open_room_request(session_id: SessionId) -> Request {
        Request::builder()
            .method(http::Method::POST)
            .header("set-cookie", format!("session_id={session_id}"))
            .uri("/room/open")
            .body(Body::empty())
            .unwrap()
    }


    pub async fn create_discussion(app: &mut Router, room_id: RoomId) -> Created {
        let response = ServiceExt::<axum::extract::Request<Body>>::ready(app)
            .await
            .unwrap()
            .call(create_discussion_request(room_id))
            .await
            .unwrap();
        serde_json::from_slice::<Created>(
            &response.into_body().collect().await.unwrap().to_bytes(),
        ).unwrap()
    }


    pub fn create_discussion_request(room_id: RoomId) -> axum::http::Request<Body> {
        tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .uri(format!("/room/{room_id}/discussion/global/create"))
            .method(http::method::Method::POST)
            .header(header::SET_COOKIE, format!("session_id={}", mock_session_id()))
            .body(Body::empty())
            .unwrap()
    }
}
