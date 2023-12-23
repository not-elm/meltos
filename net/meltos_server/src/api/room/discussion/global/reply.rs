use axum::Json;

use meltos::schema::request::discussion::global::Reply;

use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn reply(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(reply): Json<Reply>,
) -> HttpResult {
    let replied = room
        .global_discussion(user_id, |exe| exe.reply(reply))
        .await?;

    Ok(replied)
}

// #[cfg(test)]
// mod tests {
//     use meltos::discussion::message::{Message, MessageText};
//     use meltos::schema::request::discussion::global::{Reply, Speak};
//     use meltos::room::UserId;
//     use meltos_tvn::file_system::mock::MockFileSystem;
//
//     use crate::api::test_util::{
//         http_create_discussion, http_open_room, http_reply, http_speak, logged_in_app,
//     };
//
//     #[tokio::test]
//     async fn return_replied_command() {
//         let (session_id, mut app) = logged_in_app().await;
//         let mock = MockFileSystem::default();
//         let room_id = http_open_room(&mut app, mock).await;
//         let created = http_create_discussion(&mut app, room_id.clone()).await;
//         let spoke = http_speak(
//             &mut app,
//             &room_id,
//             Speak {
//                 discussion_id: created.meta.id.clone(),
//                 message: MessageText::from("message"),
//             },
//         )
//         .await;
//         let replied = http_reply(
//             &mut app,
//             &room_id,
//             Reply {
//                 target_id: spoke.message.id.clone(),
//                 text: MessageText::from("reply"),
//             },
//         )
//         .await;
//
//         assert_eq!(&replied.reply_message_id, &spoke.message.id);
//         assert_eq!(
//             replied.reply.clone(),
//             Message {
//                 id: replied.reply.id,
//                 user_id: UserId::from("room"),
//                 text: MessageText::from("reply"),
//             }
//         )
//     }
// }
