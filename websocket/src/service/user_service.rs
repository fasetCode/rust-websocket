use crate::common::dto::PageVo;
use crate::dao::user_dao;
use crate::dao::user_dao::get_username;
use crate::domain::user::{UserCreate, UserLogin, UserPageListVo, UserQuery, UserUpdate};
use crate::utils::password_utils::{hash_password, verify_password};
use sqlx::PgPool;

pub async fn get_page(
    pool: &PgPool,
    query: UserQuery,
) -> Result<PageVo<UserPageListVo>, sqlx::Error> {
    // 后面可加权限 / 状态 / 聚合逻辑
    user_dao::find_page(pool, query).await
}

pub async fn create_user(pool: &PgPool, mut user: UserCreate) -> Result<u64, sqlx::Error> {
    match user_dao::get_count_username(pool, &user.username).await {
        Ok(u) => {
            if u > 0 {
                return Err(sqlx::Error::InvalidArgument("用户已存在".to_string())); // 或使用自定义错误
            }
            user.password = hash_password(&user.password).unwrap();
            user_dao::create_user(pool, user).await
        }
        Err(_) => {
            // 用户不存在，可以创建
            Err(sqlx::Error::InvalidArgument("服务器异常".to_string()))
        }
    }
}

pub async fn update_user(pool: &PgPool, user: UserUpdate) -> Result<u64, sqlx::Error> {
    user_dao::update_user(pool, user).await
}

pub async fn delete_user(pool: &PgPool, user_id: i32) -> Result<u64, sqlx::Error> {
    user_dao::delete_user(pool, user_id).await
}

use uuid::Uuid;
use crate::props::config::get_config;

pub(crate) async fn login(
    pool: &PgPool,
    user_login: UserLogin,
    redis_manager: &crate::config::redis_manager::RedisManager,
) -> Result<String, sqlx::Error> {
    match get_username(pool, &*user_login.username).await {
        Ok(user) => {
            if verify_password(&user.password, &user_login.password) {
                // UUID
                let token = Uuid::new_v4().to_string()+ &*Uuid::new_v4().to_string();
                // 存入Redis，设置有效期为24小时
                let token_key = format!("user_token:{}", token);
                let user_id_str = user.id.to_string();

                let sys_config = get_config().expect("TODO: panic message");
                println!("{:?}", sys_config);
                // 使用异步Redis方法存储token和用户信息
                if let Err(_) = redis_manager.async_set_ex(&token_key, &user_id_str, sys_config.token_ex).await {
                    // 如果Redis存储失败，返回错误
                    return Err(sqlx::Error::InvalidArgument("Token存储失败".to_string()));
                }

                Ok(token)
            } else {
                Err(sqlx::Error::InvalidArgument("账号或密码错误".to_string()))
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            // 用户不存在
            Err(sqlx::Error::InvalidArgument("账号或密码错误".to_string()))
        }
        Err(_) => Err(sqlx::Error::InvalidArgument("服务器异常".to_string())),
    }
}
