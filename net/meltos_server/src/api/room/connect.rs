use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use serde::{Deserialize, Serialize};

use meltos::room::RoomId;
use meltos::user::SessionId;

use crate::room::Rooms;

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    room_id: RoomId,
    session_token: SessionId,
}

#[tracing::instrument]
pub async fn connect(
    ws: WebSocketUpgrade,
    Query(param): Query<Param>,
    State(rooms): State<Rooms>,
) -> Response {
    todo!()
    // if let Some(client_command_receiver) = rooms
    //     .lock()
    //     .await
    //     .room_mut(&param.room_id)
    //     .map(|room| room.command_receiver())
    // {
    //     ws.on_upgrade(move |socket| {
    //         start_websocket(
    //             socket,
    //             client_command_receiver,
    //         )
    //     })
    // } else {
    //     response_not_exists_target_room(param.room_id)
    // }
}

// async fn _start_websocket(socket: WebSocket, client_command_receiver: ClientCommandReceiver) {
//     let (mut ws_tx, _) = socket.split();
//     let r1 = _send_client_commands(&mut ws_tx, client_command_receiver).await;
//     let r2 = ws_tx.close().await;
//     r1.log_if_error();
//     r2.log_if_error();
// }
//
//
// async fn _send_client_commands(
//     ws_tx: &mut SplitSink<WebSocket, Message>,
//     mut client_rx: Receiver<ClientCommand>,
// ) -> error::Result {
//     while let Ok(client_command) = client_rx.recv().await {
//         debug!("send remote command {client_command:?}");
//
//         ws_tx.send(Message::Text(client_command.as_json())).await?;
//     }
//
//     Ok(())
// }
//
//
// fn _response_not_exists_target_room(session_id: RoomId) -> Response {
//     error!("{}", error::Error::SessionNotExists(session_id.clone()));
//     Response::builder()
//         .status(StatusCode::BAD_REQUEST)
//         .body(Body::from(
//             serde_json::to_string(&json! {{
//                 "detail": error::Error::SessionNotExists(session_id).to_string()
//             }})
//                 .unwrap(),
//         ))
//         .unwrap()
// }
