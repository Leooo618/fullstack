#[derive(serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: u32,
    pub content: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateMessage {
    pub content: String,
}