use serde::{Deserialize, Serialize};

pub mod global;


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(tag = "type", content = "command", rename_all = "snake_case")]
pub enum DiscussionCmd {
    Global(global::GlobalCmd),
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::command::request::discussion::global::GlobalCmd;
    use crate::command::request::discussion::DiscussionCmd;

    #[test]
    fn global_cmd() {
        let lhs = json!(DiscussionCmd::Global(GlobalCmd::Create));
        let object = lhs.as_object().unwrap();
        assert_eq!(object.get("type"), Some(&json!("global")));
        assert_eq!(
            object.get("command"),
            Some(&json!({
                    "type" : "create"
            }))
        );
    }
}
