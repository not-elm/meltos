use meltos_util::async_trait::async_trait;
use crate::error;

#[async_trait]
pub trait UserIo{
    async fn join(&mut self) -> error::Result;


    async fn recv(&mut self) -> error::Result;


    async fn send(&mut self) -> error::Result;
}