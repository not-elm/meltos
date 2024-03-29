use async_trait::async_trait;

use meltos_core::room::RoomId;
use meltos_core::user::{SessionId, UserId};

use crate::error;

pub mod mock;
pub mod sqlite;

pub trait NewSessionIo: Sized {
    /// Roomに対応する新たなSession Ioを作成します。
    fn new(room_id: RoomId) -> error::Result<Self>;
}

#[async_trait]
pub trait SessionIo: Send + Sync {
    /// ユーザーを登録します。
    ///
    /// ユーザーIDが指定されていない場合、自動的に付与されます。
    ///
    ///
    /// # Errors
    ///
    /// - `error::Error::UserIdConflicts` : 既に登録されているユーザーIDが指定された場合
    async fn register(&self, user_id: Option<UserId>) -> error::Result<(UserId, SessionId)>;

    /// ユーザーを削除します。
    async fn unregister(&self, user_id: UserId) -> error::Result;

    /// セッションIDに対応するユーザーIDを取得します。
    ///
    /// - [`error::Error::SessionIdNotExists`] : 存在しない[`SessionId`]が指定された場合
    async fn fetch(&self, session_id: SessionId) -> error::Result<UserId>;

    /// ルーム内の現在のユーザー数を返します。
    ///
    /// ルームオーナーも含まれます。
    async fn user_count(&self) -> error::Result<u64>;
}
