use axum::body::Body;
use axum::response::Response;
use serde::Serialize;

use meltos_util::serde::SerializeJson;

pub mod login;
pub mod room;

pub type HttpResult = std::result::Result<Response, Response>;

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
    use serde::de::DeserializeOwned;
    use tower::{Service, ServiceExt};

    use meltos::command::client::discussion::global::{Closed, Created, Replied, Spoke};
    use meltos::command::client::room::Opened;
    use meltos::command::request::discussion::global::{Reply, Speak};
    use meltos::discussion::id::DiscussionId;
    use meltos::room::RoomId;
    use meltos::user::{SessionId, UserId};
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;
    use meltos_tvn::branch::BranchName;
    use meltos_tvn::file_system::mock::MockFileSystem;
    use meltos_tvn::operation::init::Init;
    use meltos_tvn::operation::push::Push;
    use meltos_util::serde::SerializeJson;

    use crate::app;

    #[async_trait]
    pub trait ResponseConvertable {
        async fn into_json(self) -> String;
    }

    #[async_trait]
    impl ResponseConvertable for Response {
        async fn into_json(self) -> String {
            let bytes = self
                .into_body()
                .collect()
                .await
                .unwrap()
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
        (
            mock_session_id(),
            app(session, MockGlobalDiscussionIo::default()),
        )
    }

    pub fn mock_session_id() -> SessionId {
        SessionId("session_id".to_string())
    }


    pub async fn http_open_room(
        app: &mut Router,
        mock: MockFileSystem,
        user_token: SessionId,
    ) -> RoomId {
        http_call::<Opened>(app, open_room_request(user_token, mock))
            .await
            .room_id
    }

    pub async fn http_create_discussion(app: &mut Router, room_id: RoomId) -> Created {
        http_call(app, create_discussion_request(room_id)).await
    }

    pub async fn http_speak(app: &mut Router, room_id: &RoomId, speak: Speak) -> Spoke {
        http_call(app, speak_request(speak, room_id)).await
    }

    pub async fn http_reply(app: &mut Router, room_id: &RoomId, reply: Reply) -> Replied {
        http_call(
            app,
            Request::builder()
                .method(http::Method::POST)
                .header(header::SET_COOKIE, "session_id=session_id")
                .header("Content-Type", "application/json")
                .uri(format!("/room/{room_id}/discussion/global/reply"))
                .body(Body::from(reply.as_json()))
                .unwrap(),
        )
            .await
    }

    pub async fn http_discussion_close(
        app: &mut Router,
        room_id: &RoomId,
        discussion_id: DiscussionId,
    ) -> Closed {
        http_call(
            app,
            Request::builder()
                .method(http::Method::DELETE)
                .header(header::SET_COOKIE, "session_id=session_id")
                .uri(format!(
                    "/room/{room_id}/discussion/global/close?discussion_id={discussion_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
            .await
    }

    pub fn open_room_request(session_id: SessionId, mock: MockFileSystem) -> Request {
        Init::new(BranchName::main(), mock.clone())
            .execute()
            .unwrap();
        let push_param = Push::new(BranchName::main(), mock)
            .create_push_param()
            .unwrap();

        Request::builder()
            .method(http::Method::POST)
            .header("set-cookie", format!("session_id={session_id}"))
            .header(header::CONTENT_TYPE, "application/json")
            .uri("/room/open")
            .body(Body::from(serde_json::to_string(&push_param).unwrap()))
            .unwrap()
    }

    pub fn create_discussion_request(room_id: RoomId) -> axum::http::Request<Body> {
        tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .uri(format!("/room/{room_id}/discussion/global/create"))
            .method(http::method::Method::POST)
            .header(
                header::SET_COOKIE,
                format!("session_id={}", mock_session_id()),
            )
            .body(Body::empty())
            .unwrap()
    }

    pub fn speak_request(speak: Speak, room_id: &RoomId) -> axum::http::Request<Body> {
        Request::builder()
            .uri(format!("/room/{}/discussion/global/speak", room_id))
            .method(http::method::Method::POST)
            .header("Content-Type", "application/json")
            .header(header::SET_COOKIE, "session_id=session_id")
            .body(Body::new(speak.as_json()))
            .unwrap()
    }

    pub async fn http_call<D: DeserializeOwned>(app: &mut Router, request: Request) -> D {
        let response = ServiceExt::<axum::extract::Request<Body>>::ready(app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        convert_body_json::<D>(response).await
    }

    async fn convert_body_json<D: DeserializeOwned>(response: Response) -> D {
        let b = response.into_body().collect().await.unwrap().to_bytes();
        println!("{:?}", String::from_utf8(b.to_vec()));
        serde_json::from_slice::<D>(&b).unwrap()
    }
}
