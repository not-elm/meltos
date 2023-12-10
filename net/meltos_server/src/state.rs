use crate::effect::{ClientOrderTx, ServerOrderTx};
use axum::extract::FromRef;
use meltos::session::SessionId;
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


#[derive(Default, Deref, Clone)]
pub struct Rooms(ArcMutex<HashMap<SessionId, (ServerOrderTx, ClientOrderTx)>>);
