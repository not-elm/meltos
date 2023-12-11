use axum::body::Body;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use futures::stream::SplitSink;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast::Receiver;

use meltos::command::client::ClientOrder;
use meltos::command::server::ServerCommand;
use meltos::session::RoomId;
use meltos::user::UserId;
use meltos_util::error::LogIfError;
use meltos_util::serde::AsBinary;

use crate::error;
use crate::room::{ClientCommandReceiver, ServerCommandSender};
use crate::room::ws::receiver::CommandReceiver;
use crate::state::Rooms;

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    room_id: RoomId,
    user_id: UserId,
}


pub async fn connect(
    ws: WebSocketUpgrade,
    Query(param): Query<Param>,
    State(rooms): State<Rooms>,
) -> Response {
    if let Some((server_command_sender, client_command_receiver)) =
        rooms.lock().await.get(&param.room_id).cloned()
    {
        ws.on_upgrade(move |socket| {
            start_websocket(
                socket,
                server_command_sender,
                client_command_receiver,
                param.user_id,
            )
        })
    } else {
        response_not_exists_target_room(param.room_id)
    }
}


async fn start_websocket(
    socket: WebSocket,
    server_command_sender: ServerCommandSender,
    client_command_receiver: ClientCommandReceiver,
    user_id: UserId,
) {
    let (ws_tx, ws_rx) = socket.split();
    let h1 = send_client_order(ws_tx, client_command_receiver.subscribe());
    let h2 = receive_commands(
        CommandReceiver(ws_rx),
        server_command_sender.clone(),
        user_id,
    );
    tokio::select! {
        r1 = h1 => r1.log_if_error(),
        r2 = h2 => r2.log_if_error()
    }
}


async fn send_client_order(
    mut ws_tx: SplitSink<WebSocket, Message>,
    mut client_rx: Receiver<ClientOrder>,
) -> error::Result {
    while let Ok(order) = client_rx.recv().await {
        ws_tx
            .send(Message::Binary(
                order
                    .as_binary()
                    .map_err(|_| error::Error::SerializeToBinary)?,
            ))
            .await?;
    }

    Ok(())
}


async fn receive_commands(
    mut command_receiver: CommandReceiver,
    server_command_sender: ServerCommandSender,
    user_id: UserId,
) -> error::Result {
    while let Ok(command) = command_receiver.recv().await {
        server_command_sender.send(ServerCommand {
            from: user_id.clone(),
            command,
        })?;
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


#[cfg(test)]
mod tests {
    use std::future::IntoFuture;
    use std::net::SocketAddr;

    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::app;

    #[tokio::test]
    async fn integration_test() {
        let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000)))
            .await
            .unwrap();
        let app = app();
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/host/create")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let session_id = response.into_body().collect().await.unwrap().to_bytes();

        tokio::spawn(axum::serve(listener, app).into_future());


        let (_socket, _response) = tokio_tungstenite::connect_async(
            "ws://localhost:3000/host/connect?session_id=311&user_id=host",
        )
            .await
            .unwrap();
    }
}
