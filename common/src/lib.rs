use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
pub struct UserInfo {
    pub email: String,
    pub is_logged_in: bool,
    pub is_admin: bool,
    pub assigned_number: Option<i32>,
}

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct ServerSentData {
    pub selected_number: Option<i32>,
    pub assigned_number: Option<i32>,
}
