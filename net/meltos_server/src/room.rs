use log::info;
use tokio::sync::broadcast::{Receiver, Sender};

use meltos::command::client::ClientCommand;
use meltos::command::server::ServerCommand;
use meltos::session::RoomId;
use meltos::thread::io::global::mock::MockGlobalThreadIo;
use meltos_util::error::LogIfError;
use crate::room::executor::ServerOrderExecutor;

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
    let global_thread_io = MockGlobalThreadIo::default();
    while let Ok(order) = server_rx.recv().await {
        let executor = ServerOrderExecutor::new(room_id.clone(), order.from.clone(), &global_thread_io);
        info!("DDDDDDDD");
        if let Some(client_command) = executor.execute(order.command).await? {
            info!("FFFFFFFF");
            client_command_sender.send(client_command)?;
        }
    }

    Ok(())
}







