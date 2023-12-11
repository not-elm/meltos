use serde::{Deserialize, Serialize};

pub mod global;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case", content = "command")]
pub enum DiscussionCmd {
    Global(global::GlobalCmd)
}



#[cfg(test)]
mod tests{
    use serde_json::json;
    use crate::command::client::discussion::{DiscussionCmd, global};
    use crate::discussion::structs::DiscussionMeta;
    use crate::discussion::structs::id::DiscussionId;
    use crate::user::UserId;

    #[test]
    fn global_create(){
        let cmd = global::GlobalCmd::Created {
            meta: DiscussionMeta{
                id: DiscussionId::new(),
                creator: UserId::from("user")
            }
        };
        let json = json!(DiscussionCmd::Global(cmd.clone()));
        let m = json.as_object().unwrap();
        assert_eq!(m.get("type"), Some(&json!("global")));
        assert_eq!(m.get("command"), Some(&json!(cmd)));
    }
}