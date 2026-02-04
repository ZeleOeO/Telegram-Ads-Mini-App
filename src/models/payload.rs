use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddChannelPayload {
    pub username: String,
}
