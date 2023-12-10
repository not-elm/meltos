use meltos::order::client::ClientOrder;
use meltos::order::ServerOrder;
use tokio::sync::broadcast::{Receiver, Sender};


pub type ServerOrderTx = Sender<ServerOrder>;

pub type ClientOrderTx = Sender<ClientOrder>;


pub fn create_effect(capacity: usize) -> (ServerOrderTx, ClientOrderTx) {
    let (server_tx, mut server_rx) = tokio::sync::broadcast::channel::<ServerOrder>(capacity);
    let (client_tx, _client_rx) = tokio::sync::broadcast::channel::<ClientOrder>(capacity);


    tokio::spawn(async move { while let Ok(_order) = server_rx.recv().await {} });

    (server_tx, client_tx)
}
