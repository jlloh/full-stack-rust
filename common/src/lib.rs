use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub is_logged_in: bool,
}
