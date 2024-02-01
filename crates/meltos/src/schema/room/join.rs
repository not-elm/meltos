use crate::discussion::DiscussionBundle;
use crate::user::{SessionId, UserId};
use meltos_tvc::io::bundle::Bundle;
use serde::{Deserialize, Serialize};

/// ルームへの参加リクエストを表します。
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Join {
    /// ユーザーID　指定しない場合はサーバ側で割り当てられます。
    /// user_idはルームごとに一意である必要があり、既に使用されているIDを指定した場合はエラーが返されます。
    pub user_id: Option<UserId>,
}

/// ルームへの参加が正常に完了したことを表します。
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Joined {
    /// ルーム内でのユーザーID
    pub user_id: UserId,

    /// リクエスト送信時などに使用されるセッションID
    pub session_id: SessionId,

    /// バンドル化されたTVCリポジトリ
    pub bundle: Bundle,

    /// バンドル化されたディスカッション情報
    pub discussions: Vec<DiscussionBundle>,
}
