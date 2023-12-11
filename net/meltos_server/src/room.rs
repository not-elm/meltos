use tokio::sync::broadcast::{Receiver, Sender};

use meltos::command::client::ClientCommand;
use meltos::command::server::ServerCommand;
use meltos::discussion::io::global::mock::MockGlobalDiscussionIo;
use meltos::room::RoomId;
use meltos_util::error::LogIfError;

use crate::error;
use crate::room::executor::ServerCommandExecutor;
use crate::state::Rooms;

mod executor;
pub mod ws;

pub type ServerCommandSender = Sender<ServerCommand>;

pub type ClientCommandReceiver = Sender<ClientCommand>;


pub async fn room_effect(rooms: Rooms, room_id: RoomId, capacity: usize) -> error::Result {
    let (server_tx, server_rx) = tokio::sync::broadcast::channel::<ServerCommand>(capacity);
    let (client_command_sender, _) = tokio::sync::broadcast::channel::<ClientCommand>(capacity);
    insert_room(
        &rooms,
        room_id.clone(),
        server_tx,
        client_command_sender.clone(),
    )
    .await?;

    tokio::spawn(async move {
        spawn_room_effect(server_rx, client_command_sender, room_id.clone())
            .await
            .log_if_error();

        rooms.lock().await.remove(&room_id);
    });

    Ok(())
}


async fn insert_room(
    rooms: &Rooms,
    room_id: RoomId,
    server_tx: Sender<ServerCommand>,
    client_tx: Sender<ClientCommand>,
) -> error::Result {
    let mut rooms = rooms.lock().await;
    if rooms
        .insert(room_id.clone(), (server_tx, client_tx))
        .is_some()
    {
        Err(error::Error::RoomCreate(room_id))
    } else {
        Ok(())
    }
}


async fn spawn_room_effect(
    mut server_rx: Receiver<ServerCommand>,
    client_command_sender: Sender<ClientCommand>,
    room_id: RoomId,
) -> crate::error::Result {
    let global_thread_io = MockGlobalDiscussionIo::default();
    while let Ok(server_command) = server_rx.recv().await {
        let executor = ServerCommandExecutor::new(
            room_id.clone(),
            server_command.from.clone(),
            &global_thread_io,
        );
        if let Some(client_command) = executor.execute(server_command.command).await? {
            client_command_sender.send(client_command).map_err(|_|error::Error::SendClientOrder)?;
        }
    }

    Ok(())
}
