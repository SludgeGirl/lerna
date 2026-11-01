use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Messages {
    AddInstance { name: String },
    DeleteInstance { name: String },

    Status { namespace: Option<String> },

    Up { name: Option<String> },
    Down { name: Option<String> },
}
