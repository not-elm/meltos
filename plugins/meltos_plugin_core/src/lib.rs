mod request;
mod request;

use std::fmt::Display;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use crate::request::UserRequest;

#[async_trait]
pub trait RoomOwnerPlugin {
    async fn on_request(&mut self, request: UserRequest) -> Option<Vec<OwnerResponse>>;
}


#[async_trait]
pub trait RoomUserPlugin<R: DeserializeOwned> {
    type Error: Display;
    async fn execute(&mut self, command: R) -> Result<(), Self::Error>;
}

