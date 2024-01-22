use axum::extract::FromRef;

use crate::room::Rooms;
use crate::state::config::AppConfigs;

pub mod config;

#[derive(Clone, Default)]
pub struct AppState {
    pub(crate) rooms: Rooms,
    pub(crate) configs: AppConfigs,
}

impl AppState {
    #[inline(always)]
    pub fn new() -> AppState {
        Self {
            rooms: Rooms::default(),
            configs: AppConfigs::default(),
        }
    }
}

impl FromRef<AppState> for Rooms {
    #[inline(always)]
    fn from_ref(input: &AppState) -> Self {
        input.rooms.clone()
    }
}

impl FromRef<AppState> for AppConfigs {
    #[inline(always)]
    fn from_ref(input: &AppState) -> Self {
        input.configs
    }
}
