use std::time::Duration;

use serde::{Deserialize, Serialize};

use meltos_tvc::io::bundle::Bundle;

use crate::room::RoomId;
use crate::user::{SessionId, UserId};

/// POST: `room/open`で送られるbodyデータ
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Open {
    /// ルームの生存期間(秒)
    ///
    /// 指定されていない場合や上限値を超えた値が指定された場合は、サーバ側で設定された生存期間の上限値になります。
    pub lifetime_secs: Option<u64>,

    /// ルームの定員
    ///
    /// 指定されていない場合や上限値を超えた値が指定された場合は、サーバ側で設定された定員の上限値になります。
    pub user_limits: Option<u64>,

    /// TVCのバンドル情報
    pub bundle: Option<Bundle>,
}

impl Open {
    #[inline]
    pub const fn new(
        lifetime_secs: Option<u64>,
        user_limits: Option<u64>,
        bundle: Option<Bundle>,
    ) -> Self {
        Self {
            lifetime_secs,
            user_limits,
            bundle,
        }
    }

    #[inline(always)]
    pub fn lifetime_duration(&self, limit_room_life_time_sec: u64) -> Duration {
        self.lifetime_secs
            .map(|life_time| Duration::from_secs(life_time.min(limit_room_life_time_sec)))
            .unwrap_or(Duration::from_secs(limit_room_life_time_sec))
    }

    #[inline(always)]
    pub fn get_capacity(&self, max_user_limits: u64) -> u64 {
        let capacity = self
            .user_limits
            .map(|limits| limits.min(max_user_limits))
            .unwrap_or(max_user_limits);
        capacity.max(1)
    }
}

/// POST: `room/open`の正常レスポンスデータ
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opened {
    /// ユーザーID
    ///
    /// 現状`owner`固定
    pub user_id: UserId,

    /// サーバ側で作成されたRoom Id
    pub room_id: RoomId,

    /// サーバ側で作成されたセッションID
    pub session_id: SessionId,

    /// ルームの定員
    pub capacity: u64,
}

#[cfg(test)]
mod tests {
    use crate::schema::room::Open;

    #[test]
    fn return_specified_lifetime_not_exceed_limits() {
        let open = Open {
            bundle: None,
            user_limits: None,
            lifetime_secs: Some(59),
        };
        let lifetime = open.lifetime_duration(60);
        assert_eq!(lifetime.as_secs(), 59);
    }

    #[test]
    fn return_limit_lifetime_if_exceed_limits() {
        let open = Open {
            user_limits: None,
            bundle: None,
            lifetime_secs: Some(61),
        };
        let lifetime = open.lifetime_duration(60);
        assert_eq!(lifetime.as_secs(), 60);
    }

    #[test]
    fn return_limit_lifetime_if_not_specified() {
        let open = Open {
            user_limits: None,
            bundle: None,
            lifetime_secs: None,
        };
        let lifetime = open.lifetime_duration(60);
        assert_eq!(lifetime.as_secs(), 60);
    }
}
