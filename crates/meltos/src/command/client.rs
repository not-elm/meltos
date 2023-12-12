use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::error;

pub mod discussion;
pub mod room;

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(tag = "type", rename_all = "snake_case", content = "command")]
pub enum ClientCommand {
    Discussion(discussion::DiscussionCmd),
}


impl TryFrom<Message> for ClientCommand {
    type Error = error::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => Ok(serde_json::from_str(&text)?),
            _ => Err(error::Error::SerializeClientCommand),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::command::client::ClientCommand;
    use serde_json::json;

    use crate::command::client::discussion::{global, DiscussionCmd};
    use crate::discussion::id::DiscussionId;
    use crate::discussion::DiscussionMeta;
    use crate::user::UserId;

    #[test]
    fn create_global_discussion() {
        let cmd = global::GlobalCmd::Created {
            meta: DiscussionMeta {
                id: DiscussionId::new(),
                creator: UserId::from("user"),
            },
        };
        let cmd = DiscussionCmd::Global(cmd);
        let json = json!(ClientCommand::Discussion(cmd.clone()));
        let m = json.as_object().unwrap();
        assert_eq!(m.get("type"), Some(&json!("discussion")));
        assert_eq!(m.get("command"), Some(&json!(cmd)));
    }
}
