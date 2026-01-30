use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/**
 * 用户分页列表
 */
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct UserPageListVo {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/**
 * 用户查询参数
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserQuery {
    pub page: i64,
    pub page_size: i64,
}


/**
 * 用户添加
 */
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct UserCreate {
    pub username: String,
    pub password: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

// 登录
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

/**
 * 用户修改
 */
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct UserUpdate {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}


#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct User{
    pub id: i64,
    pub password: String,
    pub username: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}