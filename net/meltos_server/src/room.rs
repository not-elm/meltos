
use tokio::sync::broadcast::{Receiver, Sender};

use meltos::command::client::ClientCommand;
use meltos::command::server::ServerCommand;
use meltos::discussion::io::global::mock::MockGlobalDiscussionIo;
use meltos::room::RoomId;
use meltos_util::error::LogIfError;
use crate::room::executor::ServerCommandExecutor;

pub mod ws;
mod executor;

pub type ServerCommandSender = Sender<ServerCommand>;

pub type ClientCommandReceiver = Sender<ClientCommand>;


pub fn room_effect(
    room_id: RoomId,
    capacity: usize,
) -> (ServerCommandSender, ClientCommandReceiver) {
    let (server_tx, server_rx) = tokio::sync::broadcast::channel::<ServerCommand>(capacity);
    let (client_command_sender, _) = tokio::sync::broadcast::channel::<ClientCommand>(capacity);

    let client_command_sender2 = client_command_sender.clone();
    tokio::spawn(async move {
        spawn_room_effect(server_rx, client_command_sender2, room_id)
            .await
            .log_if_error();
    });

    (server_tx, client_command_sender)
}


async fn spawn_room_effect(
    mut server_rx: Receiver<ServerCommand>,
    client_command_sender: Sender<ClientCommand>,
    room_id: RoomId,
) -> crate::error::Result {
    let global_thread_io = MockGlobalDiscussionIo::default();
    while let Ok(server_command) = server_rx.recv().await {
        let executor = ServerCommandExecutor::new(room_id.clone(), server_command.from.clone(), &global_thread_io);
        if let Some(client_command) = executor.execute(server_command.command).await? {
            client_command_sender.send(client_command)?;
        }
    }

    Ok(())
}







