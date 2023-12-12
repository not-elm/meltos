use axum::Json;

use meltos::command::request::discussion::global::Speak;

use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn speak(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(speak): Json<Speak>,
) -> HttpResult {
    let spoke = room
        .global_discussion(user_id, move |exe| exe.speak(speak))
        .await?;

    Ok(spoke)
}


#[cfg(test)]
mod tests {
    use meltos::command::request::discussion::global::Speak;
    use meltos::discussion::message::{MessageNo, MessageText};

    use crate::api::test_util::{
        http_create_discussion, http_open_room, http_speak, logged_in_app,
    };

    #[tokio::test]
    async fn return_spoke() {
        let (session_id, mut app) = logged_in_app().await;
        let room_id = http_open_room(&mut app, session_id.clone()).await;
        let created = http_create_discussion(&mut app, room_id.clone()).await;
        let spoke = http_speak(
            &mut app,
            &room_id,
            Speak {
                discussion_id: created.meta.id.clone(),
                message: MessageText::from("Message"),
            },
        )
        .await;

        assert_eq!(&spoke.message.no, &MessageNo::default());
        assert_eq!(&spoke.message.text, &MessageText::from("Message"));
        assert_eq!(&spoke.discussion_id, &created.meta.id);
    }
}
