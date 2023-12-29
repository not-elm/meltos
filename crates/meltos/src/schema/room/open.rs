use std::time::Duration;

use serde::{Deserialize, Serialize};

use meltos_tvn::io::bundle::Bundle;

use crate::room::RoomId;
use crate::user::{SessionId, UserId};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Open {
    pub user_id: Option<UserId>,
    pub life_time_secs: Option<u64>,
    pub bundle: Option<Bundle>,
}

impl Open {
    #[inline]
    pub const fn new(
        user_id: Option<UserId>,
        life_time_minute: Option<u64>,
        bundle: Option<Bundle>,
    ) -> Self {
        Self {
            user_id,
            life_time_secs: life_time_minute,
            bundle,
        }
    }


    pub fn life_time_duration(&self) -> Duration {
        self.life_time_secs
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(60 * 60 * 6))
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
    fn duration() {
        let param = Open::new(None, Some(30), None);
        let duration = param.life_time_duration();
        assert_eq!(duration.as_secs(), 30);
    }
}