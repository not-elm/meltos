use serde::Serialize;

pub trait SerializeJson {
    fn as_json(&self) -> serde_json::Result<String>;
}


impl<S: Serialize> SerializeJson for S {
    fn as_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
