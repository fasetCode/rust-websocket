use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct ApplicationUse{
    pub id: i64,
    pub app_id: String,
    pub token: String,
    pub app_auth_url: String,
    pub app_callback_message: String
}