use tokio::sync::broadcast::{Receiver, Sender};

use meltos::command::client::ClientOrder;
use meltos::command::server::ServerCommand;
use meltos::session::RoomId;
use meltos::thread::io::global::mock::MockGlobalThreadIo;
use meltos_util::error::LogIfError;
use crate::room::executor::ServerOrderExecutor;

pub mod ws;
mod executor;

pub type ServerCommandSender = Sender<ServerCommand>;

pub type ClientCommandReceiver = Sender<ClientOrder>;


pub fn room_effect(
    room_id: RoomId,
    capacity: usize,
) -> (ServerCommandSender, ClientCommandReceiver) {
    let (server_tx, server_rx) = tokio::sync::broadcast::channel::<ServerCommand>(capacity);
    let (client_order_tx, _) = tokio::sync::broadcast::channel::<ClientOrder>(capacity);

    let client_order_tx2 = client_order_tx.clone();
    tokio::spawn(async move {
        spawn_room_effect(server_rx, client_order_tx2, room_id)
            .await
            .log_if_error();
    });

    (server_tx, client_order_tx)
}


async fn spawn_room_effect(
    mut server_rx: Receiver<ServerCommand>,
    client_command_sender: Sender<ClientOrder>,
    room_id: RoomId,
) -> crate::error::Result {
    let global_thread_io = MockGlobalThreadIo::default();
    while let Ok(order) = server_rx.recv().await {
        let executor = ServerOrderExecutor::new(room_id.clone(), order.from.clone(), &global_thread_io);
        if let Some(client_order) = executor.execute(order.command).await? {
            client_command_sender.send(client_order)?;
        }
    }

    Ok(())
}







