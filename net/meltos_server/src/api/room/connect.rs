use axum::body::Body;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast::Receiver;
use tracing::{debug, error};

use meltos::command::client::ClientCommand;
use meltos::room::RoomId;
use meltos::user::UserToken;
use meltos_util::error::LogIfError;
use meltos_util::serde::SerializeJson;

use crate::error;
use crate::room::{ClientCommandReceiver, Rooms};

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    room_id: RoomId,
    session_token: UserToken,
}


#[tracing::instrument]
pub async fn connect(
    ws: WebSocketUpgrade,
    Query(param): Query<Param>,
    State(rooms): State<Rooms>,
) -> Response {
    if let Some(client_command_receiver) = rooms
        .lock()
        .await
        .get(&param.room_id)
        .map(|room| room.command_receiver())
    {
        ws.on_upgrade(move |socket| {
            start_websocket(
                socket,
                client_command_receiver,
            )
        })
    } else {
        response_not_exists_target_room(param.room_id)
    }
}


async fn start_websocket(
    socket: WebSocket,
    client_command_receiver: ClientCommandReceiver,
) {
    let (mut ws_tx, _) = socket.split();
    let r1 = send_client_commands(&mut ws_tx, client_command_receiver).await;
    let r2 = ws_tx.close().await;
    r1.log_if_error();
    r2.log_if_error();
}


async fn send_client_commands(
    ws_tx: &mut SplitSink<WebSocket, Message>,
    mut client_rx: Receiver<ClientCommand>,
) -> error::Result {
    while let Ok(client_command) = client_rx.recv().await {
        debug!("send client command {client_command:?}");

        ws_tx.send(Message::Text(client_command.as_json()?)).await?;
    }

    Ok(())
}


fn response_not_exists_target_room(session_id: RoomId) -> Response {
    error!("{}", error::Error::SessionNotExists(session_id.clone()));
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(
            serde_json::to_string(&json! {{
                "detail": error::Error::SessionNotExists(session_id).to_string()
            }})
                .unwrap(),
        ))
        .unwrap()
}


