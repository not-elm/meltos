use serde::{Deserialize, Serialize};

use crate::discussion::id::DiscussionId;
use crate::discussion::message::Message;
use crate::discussion::reply::Reply;
use crate::discussion::DiscussionMeta;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Created {
    pub meta: DiscussionMeta,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Spoke {
    pub discussion_id: DiscussionId,
    pub message: Message,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type", content = "command")]
pub enum GlobalCmd {
    Created {
        meta: DiscussionMeta,
    },

    Spoke {
        discussion_id: DiscussionId,
        message: Message,
    },

    Replied {
        discussion_id: DiscussionId,
        reply: Reply,
    },

    Closed {
        discussion_id: DiscussionId,
    },
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::command::client::discussion::global::GlobalCmd;
    use crate::discussion::id::DiscussionId;
    use crate::discussion::DiscussionMeta;
    use crate::user::UserId;

    #[test]
    fn created() {
        let meta = DiscussionMeta {
            id: DiscussionId::new(),
            creator: UserId::from("user"),
        };
        let json = json!(GlobalCmd::Created { meta: meta.clone() });
        let m = json.as_object().unwrap();
        assert_eq!(m.get("type"), Some(&json!("created")));
        assert_eq!(
            m.get("command"),
            Some(&json!({
                "meta" : meta
            }))
        );
    }
}
