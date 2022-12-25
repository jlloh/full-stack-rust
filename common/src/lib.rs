use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash)]
pub struct UserInfo {
    pub email: String,
    pub is_logged_in: bool,
    pub is_admin: bool,
}
