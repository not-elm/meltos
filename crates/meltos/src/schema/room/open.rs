use std::time::Duration;

use serde::{Deserialize, Serialize};

use meltos_tvc::io::bundle::Bundle;

use crate::room::RoomId;
use crate::user::{SessionId, UserId};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Open {
    pub lifetime_secs: Option<u64>,
    pub user_id: Option<UserId>,
    pub bundle: Option<Bundle>,
}

impl Open {
    #[inline]
    pub const fn new(
        user_id: Option<UserId>,
        lifetime_secs: Option<u64>,
        bundle: Option<Bundle>,
    ) -> Self {
        Self {
            user_id,
            lifetime_secs,
            bundle,
        }
    }

    pub fn lifetime_duration(&self, limit_room_life_time_sec: u64) -> Duration {
        self.lifetime_secs
            .map(|life_time| {
                Duration::from_secs(life_time.min(limit_room_life_time_sec))
            })
            .unwrap_or(Duration::from_secs(limit_room_life_time_sec))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opened {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub session_id: SessionId,
}

#[cfg(test)]
mod tests {
    use crate::schema::room::Open;

    #[test]
    fn return_specified_lifetime_not_exceed_limits() {
        let open = Open{
            bundle: None,
            user_id: None,
            lifetime_secs: Some(59)
        };
        let lifetime = open.lifetime_duration(60);
        assert_eq!(lifetime.as_secs(), 59);
    }

    #[test]
    fn return_limit_lifetime_if_exceed_limits(){
        let open = Open{
            user_id: None,
            bundle: None,
            lifetime_secs: Some(61)
        };
        let lifetime = open.lifetime_duration(60);
        assert_eq!(lifetime.as_secs(), 60);
    }

    #[test]
    fn return_limit_lifetime_if_not_specified(){
        let open = Open{
            user_id: None,
            bundle: None,
            lifetime_secs: None
        };
        let lifetime = open.lifetime_duration(60);
        assert_eq!(lifetime.as_secs(), 60);
    }
}


