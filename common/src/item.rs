use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quantity: u8,
    pub icon: String,
}
