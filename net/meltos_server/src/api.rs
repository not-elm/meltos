use axum::body::Body;
use axum::response::Response;
use serde::Serialize;

use meltos_util::serde::SerializeJson;

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
    use axum::http::{header, StatusCode};
    use axum::response::Response;
    use http_body_util::BodyExt;
    use serde::de::DeserializeOwned;
    use tower::{Service, ServiceExt};

    use meltos::discussion::id::DiscussionId;
    use meltos::room::RoomId;
    use meltos::schema::request::discussion::global::{Reply, Speak};
    use meltos::schema::request::room::{Join, Open};
    use meltos::schema::response::discussion::global::{Closed, Created, Replied, Spoke};
    use meltos::schema::response::room::Opened;
    use meltos::user::{SessionId, UserId};
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;
    use meltos_tvn::branch::BranchName;
    use meltos_tvn::file_system::mock::MockFileSystem;
    use meltos_tvn::io::bundle::Bundle;
    use meltos_tvn::operation::init::Init;
    use meltos_tvn::operation::push::{Push, Pushable};
    use meltos_util::serde::SerializeJson;

    use crate::app;

    pub struct MockServerClient<'a> {
        app: &'a mut Router,
        room_id: RoomId,
        session_id: SessionId,
    }

    impl<'a> MockServerClient<'a> {
        pub fn new(
            app: &'a mut Router,
            room_id: RoomId,
            session_id: SessionId,
        ) -> MockServerClient<'a> {
            Self {
                app,
                room_id,
                session_id,
            }
        }
    }

    unsafe impl<'a> Send for MockServerClient<'a> {}

    unsafe impl<'a> Sync for MockServerClient<'a> {}


    #[async_trait]
    impl<'a> Pushable<()> for MockServerClient<'a> {
        type Error = std::io::Error;

        async fn push(&mut self, bundle: Bundle) -> std::io::Result<()> {
            let response = http_call(
                self.app,
                Request::builder()
                    .header(
                        header::SET_COOKIE,
                        format!("session_id={}", self.session_id),
                    )
                    .header(header::CONTENT_TYPE, "application/json")
                    .method(http::method::Method::POST)
                    .uri(format!("/room/{}/tvn/push", self.room_id))
                    .body(Body::from(serde_json::to_string(&bundle).unwrap()))
                    .unwrap(),
            )
                .await;
            assert_eq!(response.status(), StatusCode::OK);
            Ok(())
        }
    }

    #[async_trait]
    pub trait ResponseConvertable {
        async fn into_json(self) -> String;

        async fn deserialize<D: DeserializeOwned>(self) -> D;
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

        async fn deserialize<D: DeserializeOwned>(self) -> D {
            convert_body_json(self).await
        }
    }

    pub async fn logged_in_app() -> (SessionId, Router) {
        let session = MockUserSessionIo::default();
        let (_, session_id) = session.register(Some(UserId::from("owner"))).await.unwrap();
        (session_id, app(session, MockGlobalDiscussionIo::default()))
    }

    pub fn mock_session_id() -> SessionId {
        SessionId("session_id".to_string())
    }

    pub fn owner_session_id() -> SessionId {
        SessionId("owner".to_string())
    }

    #[allow(unused)]
    pub fn user_session_id() -> SessionId {
        SessionId("room".to_string())
    }


    pub async fn http_open_room(app: &mut Router, mock: MockFileSystem) -> RoomId {
        http_call_with_deserialize::<Opened>(app, open_room_request(mock))
            .await
            .room_id
    }


    pub async fn http_fetch(app: &mut Router, room_id: &RoomId, session_id: &SessionId) -> Bundle {
        http_call_with_deserialize::<Bundle>(
            app,
            Request::builder()
                .header(header::SET_COOKIE, format!("session_id={session_id}"))
                .uri(format!("/room/{room_id}/tvn/fetch"))
                .body(Body::empty())
                .unwrap(),
        )
            .await
    }

    pub async fn http_create_discussion(app: &mut Router, room_id: RoomId) -> Created {
        http_call_with_deserialize(app, create_discussion_request(room_id)).await
    }

    pub async fn http_speak(app: &mut Router, room_id: &RoomId, speak: Speak) -> Spoke {
        http_call_with_deserialize(app, speak_request(speak, room_id)).await
    }

    pub async fn http_reply(app: &mut Router, room_id: &RoomId, reply: Reply) -> Replied {
        http_call_with_deserialize(
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
        http_call_with_deserialize(
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

    pub fn open_room_request(mock: MockFileSystem) -> Request {
        Init::new(BranchName::main(), mock.clone())
            .execute()
            .unwrap();
        let bundle = Push::new(BranchName::main(), mock)
            .create_push_bundle()
            .unwrap();

        Request::builder()
            .method(http::Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .uri("/room/open")
            .body(Body::from(
                serde_json::to_string(&Open {
                    user_id: Some(UserId::from("owner")),
                    bundle,
                })
                    .unwrap(),
            ))
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

    pub async fn http_join(
        app: &mut Router,
        room_id: &RoomId,
        user_id: Option<UserId>,
    ) -> Response {
        http_call(
            app,
            Request::builder()
                .uri(format!("/room/{room_id}/join"))
                .header("Content-Type", "application/json")
                .method(http::method::Method::POST)
                .body(Body::from(
                    serde_json::to_string(&Join {
                        user_id,
                    })
                        .unwrap(),
                ))
                .unwrap(),
        )
            .await
    }

    pub async fn http_call(app: &mut Router, request: Request) -> Response {
        ServiceExt::<axum::extract::Request<Body>>::ready(app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap()
    }


    pub async fn http_call_with_deserialize<D: DeserializeOwned>(
        app: &mut Router,
        request: Request,
    ) -> D {
        let response = http_call(app, request).await;
        println!("{response:?}");
        convert_body_json::<D>(response).await
    }

    pub async fn convert_body_json<D: DeserializeOwned>(response: Response) -> D {
        let b = response.into_body().collect().await.unwrap().to_bytes();
        println!("{:?}", String::from_utf8(b.to_vec()));
        serde_json::from_slice::<D>(&b).unwrap()
    }
}
