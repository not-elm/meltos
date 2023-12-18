use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Display, Sha1};

#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Sha1)]
pub struct UserId(String);
impl_string_new_type!(UserId);

#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Display, Sha1)]
pub struct SessionId(pub String);


#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Display)]
pub struct Exp(i64);

impl Exp {
    #[inline]
    pub fn as_date(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.0, 0).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use chrono::Utc;
    use crate::user::Exp;

    #[test]
    fn convert_date() {
        let date = Utc::now();
        let timestamp = date.timestamp();

        assert_eq!(Exp(timestamp).as_date(), date);
    }
}