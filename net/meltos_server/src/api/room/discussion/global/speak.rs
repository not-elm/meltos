use axum::Json;
use meltos::channel::{ChannelMessage, MessageData};

use meltos::schema::discussion::global::Speak;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn speak(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(speak): Json<Speak>,
) -> HttpResult {
    let spoke = room
        .global_discussion(user_id.clone(), move |exe| exe.speak(speak))
        .await?;

    room.send_all_users(ChannelMessage {
        from: user_id,
        message: MessageData::DiscussionSpoke(spoke.clone()),
    })
    .await?;

    Ok(spoke.as_success_response())
}

#[cfg(test)]
mod tests {
    use meltos::discussion::message::MessageText;
    use meltos::schema::discussion::global::Speak;
    use meltos_tvn::file_system::mock::MockFileSystem;

    use crate::api::test_util::{
        http_create_discussion, http_open_room, http_speak, logged_in_app,
    };

    #[tokio::test]
    async fn return_spoke() {
        let (session_id, mut app) = logged_in_app().await;
        let mock = MockFileSystem::default();
        let room_id = http_open_room(&mut app, mock).await;
        let created =
            http_create_discussion(&mut app, &session_id, "title".to_string(), room_id.clone())
                .await;
        let spoke = http_speak(
            &mut app,
            &room_id,
            &session_id,
            Speak {
                discussion_id: created.meta.id.clone(),
                message: MessageText::from("Message"),
            },
        )
        .await;

        assert_eq!(&spoke.message.text, &MessageText::from("Message"));
        assert_eq!(&spoke.discussion_id, &created.meta.id);
    }
}
