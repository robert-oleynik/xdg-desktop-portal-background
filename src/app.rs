use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct App {
    pub id: String,
    pub autostart: bool,
    pub cmd: Vec<String>,
    pub flags: u32,
}
