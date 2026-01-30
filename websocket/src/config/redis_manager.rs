use redis::{AsyncCommands, Client, Commands, FromRedisValue, RedisResult, ToRedisArgs};
use redis::aio::{MultiplexedConnection, ConnectionManager};
use std::sync::Arc;
use std::time::Duration;
use crate::web_socket::app_node::SessionUser;

#[derive(Clone)]
pub struct RedisManager {
    client: Arc<Client>,
}

impl RedisManager {
    /// 创建新的 Redis 管理器
    pub fn new(url: &str) -> RedisResult<Self> {
        let client = Client::open(url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// 创建带超时配置的 Redis 管理器
    pub fn new_with_timeout(url: &str, timeout: Duration) -> RedisResult<Self> {
        let client = Client::open(url)?;
        // 这里可以设置客户端超时配置（如果需要）
        Ok(Self {
            client: Arc::new(client),
        })
    }

    // ========== 同步方法 ==========

    /// 获取同步连接
    pub fn get_connection(&self) -> RedisResult<redis::Connection> {
        self.client.get_connection()
    }

    /// 设置键值（同步）
    pub fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.set(key, value)
    }

    /// 设置键值带过期时间（同步）
    pub fn set_ex(&self, key: &str, value: &str, seconds: u64) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.set_ex(key, value, seconds)
    }

    /// 获取值（同步）
    pub fn get(&self, key: &str) -> RedisResult<String> {
        let mut conn = self.get_connection()?;
        conn.get(key)
    }

    pub fn get_not_null(&self, key: &str) -> RedisResult<String> {
        let mut conn = self.get_connection()?;
        let val: Option<String> = conn.get(key)?;
        Ok(val.unwrap_or_default())
    }

    /// 删除键（同步）
    pub fn del(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.del(key)
    }

    /// 检查键是否存在（同步）
    pub fn exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.get_connection()?;
        conn.exists(key)
    }

    /// 设置过期时间（同步）
    pub fn expire(&self, key: &str, seconds: i64) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.expire(key, seconds)
    }

    // ========== 异步方法 ==========

    /// 获取异步多路复用连接（用于多个并发操作）
    pub async fn get_multiplexed_connection(&self) -> RedisResult<MultiplexedConnection> {
        self.client.get_multiplexed_async_connection().await
    }

    /// 获取异步连接管理器（单个连接）
    pub async fn get_connection_manager(&self) -> RedisResult<ConnectionManager> {
        self.client
            .as_ref()
            .get_connection_manager()
            .await
    }

    /// 异步设置键值
    pub async fn async_set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.set(key, value).await
    }

    /// 异步设置键值带过期时间
    pub async fn async_set_ex(&self, key: &str, value: &str, seconds: u64) -> RedisResult<()> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.set_ex(key, value, seconds).await
    }

    /// 异步获取值
    pub async fn async_get(&self, key: &str) -> RedisResult<String> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.get(key).await
    }

    /// 异步删除键
    pub async fn async_del(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.del(key).await
    }

    /// 异步检查键是否存在
    pub async fn async_exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.exists(key).await
    }

    /// 异步设置过期时间
    pub async fn async_expire(&self, key: &str, seconds: i64) -> RedisResult<()> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.expire(key, seconds).await
    }

    // ========== 高级功能 ==========

    /// 发布消息到频道
    pub async fn publish(&self, channel: &str, message: &str) -> RedisResult<()> {
        let mut conn = self.get_multiplexed_connection().await?;
        conn.publish(channel, message).await
    }

    /// 订阅频道
    pub async fn subscribe(&self, channel: &str) -> RedisResult<redis::aio::PubSub> {
        let mut pubsub = self
            .client
            .as_ref()
            .get_async_pubsub()
            .await?;

        pubsub.subscribe(channel).await?;
        Ok(pubsub)
    }

    /// 获取 Redis 客户端（用于特殊操作）
    pub fn get_client(&self) -> Arc<Client> {
        self.client.clone()
    }

    /// 测试连接
    pub async fn ping(&self) -> RedisResult<String> {
        let mut conn = self.get_multiplexed_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await
    }
}