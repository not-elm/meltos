use axum::Json;

use meltos::command::request::discussion::global::Speak;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn speak(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(speak): Json<Speak>,
) -> HttpResult {
    let spoke = room.as_global_discussion_executor(user_id)
        .speak(speak)
        .await?;

    Ok(spoke.as_success_response())
}


#[cfg(test)]
mod tests {
    use axum::{http, Router};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{header, StatusCode};
    use http_body_util::BodyExt;
    use tower::{Service, ServiceExt};

    use meltos::command::client::discussion::global::Spoke;
    use meltos::command::request::discussion::global::Speak;
    use meltos::discussion::message::{MessageNo, MessageText};
    use meltos::room::RoomId;
    use meltos_util::serde::SerializeJson;

    use crate::api::test_util::{create_discussion, logged_in_app, open_room};

    #[tokio::test]
    async fn return_spoke() {
        let (session_id, mut app) = logged_in_app().await;
        let room_id = open_room(&mut app, session_id.clone()).await;
        let created = create_discussion(&mut app, room_id.clone()).await;
        let spoke = speak(&mut app, &room_id, Speak {
            discussion_id: created.meta.id.clone(),
            message: MessageText::from("Message"),
        })
            .await;

        assert_eq!(&spoke.message.no, &MessageNo::default());
        assert_eq!(&spoke.message.text, &MessageText::from("Message"));
        assert_eq!(&spoke.discussion_id, &created.meta.id);
    }


    async fn speak(app: &mut Router, room_id: &RoomId, speak: Speak) -> Spoke {
        let response = ServiceExt::<axum::extract::Request<Body>>::ready(app)
            .await
            .unwrap()
            .call(speak_request(speak, room_id))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        serde_json::from_slice::<Spoke>(
            &response.into_body().collect().await.unwrap().to_bytes(),
        ).unwrap()
    }

    fn speak_request(speak: Speak, room_id: &RoomId) -> axum::http::Request<Body> {
        Request::builder()
            .uri(format!("/room/{}/discussion/global/speak", room_id))
            .method(http::method::Method::POST)
            .header("Content-Type", "application/json")
            .header(header::SET_COOKIE, "session_id=session_id")
            .body(Body::new(speak.as_json()))
            .unwrap()
    }
}