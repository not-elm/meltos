use crate::room::{ClientCommandReceiver, ServerCommandSender};
use axum::extract::FromRef;
use meltos::room::RoomId;
use meltos_util::macros::Deref;
use meltos_util::sync::arc_mutex::ArcMutex;
use std::collections::HashMap;


#[derive(Clone, Default)]
pub struct AppState {
    rooms: Rooms,
}


impl FromRef<AppState> for Rooms {
    fn from_ref(input: &AppState) -> Self {
        input.rooms.clone()
    }
}


#[derive(Default, Deref, Clone, Debug)]
pub struct Rooms(ArcMutex<HashMap<RoomId, (ServerCommandSender, ClientCommandReceiver)>>);
