use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::error;

pub mod discussion;



#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type", rename_all = "snake_case", content = "command")]
pub enum RequestCmd {
    Discussion(discussion::DiscussionCmd),
}


impl Display for RequestCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", serde_json::to_string(self).unwrap()))
    }
}



impl TryFrom<&str> for RequestCmd {
    type Error = error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}



#[cfg(test)]
mod tests{
    use serde_json::json;
    use crate::command::request::RequestCmd;
    use crate::command::request::discussion::global::GlobalCmd;
    use crate::command::request::discussion::DiscussionCmd;

    #[test]
    fn new_global_discussion(){
        let cmd = RequestCmd::Discussion(DiscussionCmd::Global(GlobalCmd::Create));
        let json = json!(cmd);
        let m = json.as_object().unwrap();
        assert_eq!(m.get("type"), Some(&json!("discussion")));
        assert_eq!(m.get("command"), Some(&json!({
            "type" : "global",
            "command" : {
                "type" : "create"
            }
        })));
    }
}