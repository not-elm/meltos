use axum::extract::FromRef;

use crate::room::Rooms;

#[derive(Clone, Default)]
pub struct AppState {
    pub(crate) rooms: Rooms,
}

impl AppState
{
    #[inline(always)]
    pub fn new() -> AppState {
        Self {
            rooms: Rooms::default(),
        }
    }
}


impl FromRef<AppState> for Rooms {
    fn from_ref(input: &AppState) -> Self {
        input.rooms.clone()
    }
}

