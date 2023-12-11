use std::collections::HashMap;

use tokio::sync::broadcast::{Receiver, Sender};

use meltos::command::client::ClientCommand;
use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_util::macros::Deref;
use meltos_util::sync::arc_mutex::ArcMutex;

mod executor;
pub mod ws;


pub type ClientCommandSender = Sender<ClientCommand>;
pub type ClientCommandReceiver = Receiver<ClientCommand>;

#[derive(Default, Deref, Clone, Debug)]
pub struct Rooms(ArcMutex<HashMap<RoomId, Room>>);


impl Rooms {
    pub async fn insert_room(&self, room: Room) {
        let mut rooms = self.0.lock().await;
        rooms.insert(room.id.clone(), room);
    }
}


#[derive(Debug)]
pub struct Room {
    pub owner: UserId,
    pub id: RoomId,
    command_tx: ClientCommandSender,
}


impl Room {
    pub fn open(owner: UserId, capacity: usize) -> Self {
        let (command_tx, _) = tokio::sync::broadcast::channel::<ClientCommand>(capacity);
        Self {
            id: RoomId::default(),
            owner,
            command_tx,
        }
    }


    #[inline]
    pub fn command_receiver(&self) -> ClientCommandReceiver {
        self.command_tx.subscribe()
    }
}





