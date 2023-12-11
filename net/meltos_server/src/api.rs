use axum::body::Body;
use axum::response::Response;
use serde::Serialize;

use meltos_util::serde::SerializeJson;

pub mod discussion;
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
    use axum::body::Body;
    use axum::extract::Request;
    use axum::{http, Router};
    use http_body_util::BodyExt;
    use tower::{Service, ServiceExt};

    use meltos::command::client::room::Opened;
    use meltos::command::request::room::Open;
    use meltos::room::RoomId;
    use meltos::user::{UserId, UserToken};
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;
    use meltos_util::serde::SerializeJson;

    use crate::app;

    pub async fn logged_in_app() -> (UserToken, Router) {
        let session = MockUserSessionIo::default();
        session
            .register(UserToken("token".to_string()), UserId::from("user"))
            .await
            .unwrap();
        (UserToken("token".to_string()), app(session))
    }


    pub async fn open_room(app: &mut Router, user_token: UserToken) -> RoomId {
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


    pub fn open_room_request(user_token: UserToken) -> Request {
        Request::builder()
            .method(http::Method::POST)
            .header("Content-Type", "application/json")
            .uri("/room/open")
            .body(Body::from(Open { user_token }.as_json()))
            .unwrap()
    }
}
