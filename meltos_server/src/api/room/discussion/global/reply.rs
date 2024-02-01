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
        .await;

    Ok(replied.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use meltos::discussion::id::DiscussionId;
    use meltos::discussion::message::{Message, MessageId, MessageText};
    use meltos::schema::discussion::global::{Created, Reply, Speak};
    use meltos::schema::error::{DiscussionNotExistsBody, ErrorResponseBodyBase, MessageNotExistsBody};
    use meltos::schema::room::Opened;
    use meltos::user::UserId;
    use meltos_tvc::file_system::memory::MemoryFileSystem;

    use crate::api::test_util::{http_call, http_create_discussion, http_open_room, http_reply, http_speak, mock_app, reply_request, ResponseConvertable};

    #[tokio::test]
    async fn return_replied_command() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
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

    #[tokio::test]
    async fn failed_if_not_exists_discussion() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let Opened {
            session_id,
            room_id,
            ..
        } = http_open_room(&mut app, fs).await;

        let response = http_call(
            &mut app,
            reply_request(
                &room_id,
                &session_id,
                Reply {
                    discussion_id: DiscussionId("id".to_string()),
                    to: Default::default(),
                    text: MessageText("MESSAGE".to_string()),
                },
            ),
        )
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response
            .deserialize::<DiscussionNotExistsBody>()
            .await;
        assert_eq!(error, DiscussionNotExistsBody {
            base: ErrorResponseBodyBase {
                category: "discussion".to_string(),
                error_type: "DiscussionNotExists".to_string(),
                message: "discussion not exists; id: id".to_string(),
            },
            discussion_id: DiscussionId("id".to_string()),
        });
    }


    #[tokio::test]
    async fn failed_if_not_exists_message() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let Opened {
            session_id,
            room_id,
            ..
        } = http_open_room(&mut app, fs).await;

        let Created{
            meta
        } = http_create_discussion(
            &mut app,
            &session_id,
            "title".to_string(),
            room_id.clone()
        )
            .await;

        let response = http_call(
            &mut app,
            reply_request(
                &room_id,
                &session_id,
                Reply {
                    discussion_id: meta.id,
                    to: MessageId("Null".to_string()),
                    text: MessageText("MESSAGE".to_string()),
                },
            ),
        )
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response
            .deserialize::<MessageNotExistsBody>()
            .await;
        assert_eq!(error, MessageNotExistsBody {
            base: ErrorResponseBodyBase {
                category: "discussion".to_string(),
                error_type: "MessageNotExists".to_string(),
                message: "message not exists; id: Null".to_string(),
            },
            message_id: MessageId("Null".to_string())
        });
    }
}
