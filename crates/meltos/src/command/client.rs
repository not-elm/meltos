pub mod thread;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub enum ClientOrder {
    Thread(thread::ThreadOrder)
}
