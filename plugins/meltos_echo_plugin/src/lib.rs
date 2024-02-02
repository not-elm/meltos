use meltos_plugin_core::RoomUserPlugin;
use serde::{Deserialize, Serialize};

pub struct EchoPlugin;


#[derive(Debug, Serialize, Deserialize)]
pub struct Echo{
    pub message: String
}

impl RoomUserPlugin<Echo> for EchoPlugin {
    type Error = ();

    async fn execute(&mut self, command: Echo) -> Result<(), Self::Error> {
        println!("{command:?}");
        Ok(())
    }
}