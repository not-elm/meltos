use axum::Json;

use meltos::channel::{ChannelMessage, MessageData};
use meltos::schema::discussion::global::Reply;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn reply(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(reply): Json<Reply>,
) -> HttpResult {
    let replied = room
        .global_discussion(user_id.clone(), |exe| exe.reply(reply))
        .await?;

    room.send_all_users(ChannelMessage {
        from: user_id,
        message: MessageData::DiscussionReplied(replied.clone()),
    })
    .await?;

    Ok(replied.as_success_response())
}

#[cfg(test)]
mod tests {
    use meltos::discussion::message::{Message, MessageText};
    use meltos::schema::discussion::global::{Reply, Speak};
    use meltos::schema::room::Opened;
    use meltos::user::UserId;
    use meltos_tvc::file_system::mock::MockFileSystem;

    use crate::api::test_util::{
        http_create_discussion, http_open_room, http_reply, http_speak, mock_app,
    };

    #[tokio::test]
    async fn return_replied_command() {
        let mut app = mock_app();
        let fs = MockFileSystem::default();
        let Opened {
            session_id,
            room_id,
            ..
        } = http_open_room(&mut app, fs).await;
        let created =
            http_create_discussion(&mut app, &session_id, "title".to_string(), room_id.clone())
                .await;
        let spoke = http_speak(
            &mut app,
            &room_id,
            &session_id,
            Speak {
                discussion_id: created.meta.id.clone(),
                text: MessageText::from("message"),
            },
        )
        .await;
        let replied = http_reply(
            &mut app,
            &room_id,
            &session_id,
            Reply {
                discussion_id: created.meta.id.clone(),
                to: spoke.message.id.clone(),
                text: MessageText::from("reply"),
            },
        )
        .await;

        assert_eq!(&replied.to, &spoke.message.id);
        assert_eq!(
            replied.message.clone(),
            Message {
                id: replied.message.id,
                user_id: UserId::from("owner"),
                text: MessageText::from("reply"),
            }
        )
    }
}
