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
    use axum::http::StatusCode;
    use meltos::discussion::id::DiscussionId;
    use meltos::discussion::message::MessageText;
    use meltos::schema::discussion::global::Speak;
    use meltos::schema::error::{DiscussionNotExistsBody, ErrorResponseBodyBase};
    use meltos::schema::room::Opened;
    use meltos_tvc::file_system::mock::MockFileSystem;

    use crate::api::test_util::{http_call, http_create_discussion, http_open_room, http_speak, mock_app, ResponseConvertable, speak_request};

    #[tokio::test]
    async fn return_spoke() {
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
                text: MessageText::from("Message"),
            },
        )
            .await;

        assert_eq!(&spoke.message.text, &MessageText::from("Message"));
        assert_eq!(&spoke.discussion_id, &created.meta.id);
    }


    #[tokio::test]
    async fn failed_if_not_exists_discussion() {
        let mut app = mock_app();
        let fs = MockFileSystem::default();
        let Opened {
            session_id,
            room_id,
            ..
        } = http_open_room(&mut app, fs).await;

        let response = http_call(
            &mut app,
            speak_request(
                Speak {
                    discussion_id: DiscussionId("ID".to_string()),
                    text: MessageText::from("Message"),
                },
                &room_id,
                &session_id,
            )
        )
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response
            .deserialize::<DiscussionNotExistsBody>()
            .await;
        assert_eq!(error, DiscussionNotExistsBody{
            base: ErrorResponseBodyBase{
                category: "discussion".to_string(),
                error_type: "DiscussionNotExists".to_string(),
                message: "discussion not exists; id: ID".to_string()
            },
            discussion_id: DiscussionId("ID".to_string())
        });
    }
}
