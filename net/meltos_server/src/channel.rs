use axum::async_trait;

#[async_trait]
pub trait RoomSendable {
    async fn send(&self);
}
