
use meltos_util::async_trait::async_trait;
use crate::error;

#[async_trait]
pub trait HostIo{
    async fn connect(&self) -> error::Result;
}