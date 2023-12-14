use serde::Serialize;

pub trait SerializeJson {
    fn as_json(&self) -> String;
}

impl<S: Serialize> SerializeJson for S {
    fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
