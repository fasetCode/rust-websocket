use crate::config::redis_manager::RedisManager;
use crate::http::http_util::http_post;
use crate::props::config::get_config;
use crate::service::application_use_service::get_app_id;
use crate::web_socket::app_node::{AppNode, SessionUser};
use actix::{
    Actor, ActorContext, Addr, AsyncContext, Handler, Message, Running, StreamHandler, spawn,
};
use actix_web::web::Data;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub(crate) app_name: String,
    pub(crate) session_manager: SessionManager,
    pub(crate) redis: Data<RedisManager>,
    pub(crate) db: PgPool,
}

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
    app_id: Option<String>,
    user_id: Option<String>,
}

pub struct WsConn {
    state: Data<AppState>,
    #[allow(dead_code)]
    token: Option<String>,
    app_id: Option<String>,
    session_id: String,
    #[allow(dead_code)]
    client_id: Option<String>,
    user_id: Option<String>,
}

/// 全局会话管理器
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Addr<WsConn>>>>, // 存储 session_id -> token 映射
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 注册连接
    async fn add_session(&self, session_id: String, addr: Addr<WsConn>) {
        self.sessions.lock().await.insert(session_id, addr);
    }

    // 删除连接
    pub async fn remove_session(&self, session_id: &str) {
        self.sessions.lock().await.remove(session_id);
    }

    // 获取连接
    pub(crate) async fn get_session(&self, session_id: &str) -> Option<Addr<WsConn>> {
        self.sessions.lock().await.get(session_id).cloned()
    }

    // 链接是否存在
    pub async fn is_session_exists(&self, session_id: &str) -> bool {
        return self.sessions.lock().await.contains_key(session_id);
    }
}
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let query = web::Query::<WsQuery>::from_query(req.query_string()).unwrap();

    // 验证app_id和token是否传入
    if query.app_id.is_none() || query.token.is_none() {
        println!("WebSocket connection rejected: Missing app_id or token");
        return Ok(HttpResponse::Forbidden().finish());
    }

    if let Some(ref token) = query.token {
        println!("WebSocket connection with token: {}", token);
    } else {
        println!("WebSocket connection without token");
    }

    // 生成唯一的session ID
    let session_id = Uuid::new_v4().to_string();
    println!("New WebSocket connection, session ID: {}", session_id);

    ws::start(
        WsConn {
            state,
            token: query.token.clone(),
            app_id: query.app_id.clone(),
            session_id,
            client_id: None,
            user_id: query.user_id.clone(),
        },
        &req,
        stream,
    )
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct WsContext {
    code: i32,               // 状态吗
    message: Option<String>, // 错误信息
    data: Option<String>,    // 数据
    client_id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct PushRequest {
    code: i32,                       // 状态吗
    message: Option<String>,         // 错误信息
    pub(crate) data: Option<String>, // 数据
    pub(crate) client_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) app_id: Option<String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerText(pub String);

impl Handler<ServerText> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: ServerText, ctx: &mut Self::Context) {
        // 检查消息是否包含401错误码
        if msg.0.contains("\"code\":401") || msg.0.contains("\"code\": 401") {
            // 解析JSON消息以检查code字段
            if let Ok(parsed_msg) = serde_json::from_str::<serde_json::Value>(&msg.0) {
                if parsed_msg.get("code").and_then(|c| c.as_i64()) == Some(401) {
                    // 如果code是401，发送错误消息并关闭连接
                    ctx.close(None);
                    return;
                }
            }
        }else{
            debug!("Sending message to client: {}", msg.0);
            ctx.text(msg.0);
        }
    }
}

impl WsConn {
    /// 验证token并更新会话信息
    fn validate_and_update_session(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        if let Some(token) = &self.token {
            let addr = ctx.address();
            let token_clone = token.clone();
            let db = self.state.db.clone();
            let redis = self.state.redis.clone();
            let session_manager = self.state.session_manager.clone();
            let app_id = self.app_id.clone();
            let session_id = self.session_id.clone();
            let config = get_config().expect("Failed to load config");

            spawn(async move {
                debug!("Validating token: {}", token_clone.as_str());

                match get_app_id(&db, app_id.unwrap()).await {
                    Ok(app_usr) => {
                        debug!("Found app user: {:?}", app_usr);

                        let token = token_clone;
                        let app_token = app_usr.token;
                        let data = format!(r#"{{"token":"{}","appToken":"{}"}}"#, token, app_token);

                        match http_post(&app_usr.app_auth_url, &data, &[]).await {
                            Ok(response) => {
                                if let Some(code) = response.get("code").and_then(Value::as_i64) {
                                    if code == 200 {
                                        if let Some(user_id) = response
                                            .get("data")
                                            .and_then(|d| d.get("userId"))
                                            .and_then(Value::as_str)
                                        {
                                            debug!("User ID: {}", user_id);

                                            let node = AppNode::new(
                                                config.app_ip,
                                                config.port,
                                                session_id.clone(),
                                            );
                                            let redis_session_key = format!(
                                                "web:socket:app_id:{}:user:id:{}",
                                                app_usr.app_id, user_id
                                            );

                                            match redis.get(&redis_session_key) {
                                                Ok(cached_session) => {
                                                    debug!("Successfully retrieved cached session");
                                                    if let Err(e) = Self::update_existing_session(
                                                        cached_session,
                                                        node,
                                                        &redis,
                                                        &redis_session_key,
                                                        &session_manager,
                                                    )
                                                    .await
                                                    {
                                                        error!(
                                                            "Error updating existing session: {:?}",
                                                            e
                                                        );
                                                    }
                                                }
                                                Err(_) => {
                                                    debug!("Creating new session for user");
                                                    if let Err(e) = Self::create_new_session(
                                                        node,
                                                        &redis,
                                                        &redis_session_key,
                                                    )
                                                    .await
                                                    {
                                                        error!(
                                                            "Error creating new session: {:?}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // 验证失败，向连接发送关闭消息
                                        addr.do_send(ServerText(
                                            r#"{"code":401,"message":"Token validation failed"}"#
                                                .to_string(),
                                        ));
                                    }
                                } else {
                                    // 验证失败，向连接发送关闭消息
                                    addr.do_send(ServerText(r#"{"code":401,"message":"Invalid response from auth server"}"#.to_string()));
                                }
                            }
                            Err(error) => {

                                error!("Auth server error: {:?}", error);
                                addr.do_send(ServerText(
                                    r#"{"code":401,"message":"Auth server error"}"#.to_string(),
                                ));
                            }
                        };
                    }
                    Err(_) => {
                        error!("Application does not exist");
                        addr.do_send(ServerText(
                            r#"{"code":401,"message":"Invalid app_id"}"#.to_string(),
                        ));
                    }
                };
                // return Ok(code);
            });
        }
    }

    /// 更新现有会话
    async fn update_existing_session(
        cached_session: String,
        node: AppNode,
        redis: &Data<RedisManager>,
        redis_session_key: &str,
        session_manager: &SessionManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Updating existing session");

        let mut session_user: SessionUser = serde_json::from_str(&cached_session)?;

        // 清理不存在的节点
        Self::cleanup_stale_nodes(&mut session_user, session_manager).await;

        // 添加当前节点
        session_user.nodes.push(node);

        // 保存更新后的会话信息
        let updated_session = serde_json::to_string(&session_user)?;
        redis.set(redis_session_key, &updated_session)?;

        debug!("Session updated successfully");
        Ok(())
    }

    /// 清理失效的节点
    async fn cleanup_stale_nodes(session_user: &mut SessionUser, session_manager: &SessionManager) {
        let config = get_config().expect("Failed to load config");

        let mut nodes_to_remove = Vec::new();

        // 收集需要删除的节点索引
        for (index, app_node) in session_user.nodes.iter().enumerate() {
            if app_node.ip == config.app_ip && config.port == app_node.port {
                // 在同一节点上，检查会话是否存在
                if !session_manager
                    .is_session_exists(&app_node.session_id)
                    .await
                {
                    nodes_to_remove.push(index);
                }
            }
        }

        // 逆序删除节点，避免索引变化问题
        for &index in nodes_to_remove.iter().rev() {
            session_user.nodes.remove(index);
        }
    }

    /// 创建新会话
    async fn create_new_session(
        node: AppNode,
        redis: &Data<RedisManager>,
        redis_session_key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Creating new session");

        let session_user = SessionUser::new(vec![node]);
        let session_user_str = serde_json::to_string(&session_user)?;
        redis.set(redis_session_key, &session_user_str)?;

        debug!("New session created successfully");
        Ok(())
    }

    /// 注册会话到管理器
    fn register_session(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let addr = ctx.address();
        let session_id = self.session_id.clone();
        let manager = self.state.session_manager.clone();

        spawn(async move {
            manager.add_session(session_id, addr).await;
        });
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Session {} started", self.session_id);
        // app_id 与 token 没有传入不让连接
        if self.app_id.is_none() || self.token.is_none() || self.user_id.is_none() {
            warn!("WebSocket connection closed: Missing app_id or token");
            ctx.close(None);
            return;
        }

        // 验证 token 并更新 Redis 会话信息
        self.validate_and_update_session(ctx);
        self.register_session(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!("Session {} stopping", self.session_id);

        // self.session_id 移除 redis

        let app_id = self.app_id.clone();
        let user_id = self.user_id.clone();
        let redis = self.state.redis.clone();

        // let redis = self.state.redis.clone();
        // redis.get()
        //
        let redis_session_key = format!(
            "web:socket:app_id:{}:user:id:{}",
            &app_id.unwrap(),
            &user_id.unwrap()
        );
        match redis.get(&redis_session_key) {
            Ok(cached_session) => {
                let mut session_user: SessionUser = serde_json::from_str(&cached_session).unwrap();
                for (index, app_node) in session_user.nodes.iter().enumerate() {
                    if app_node.session_id == self.session_id {
                        session_user.nodes.remove(index);
                        break;
                    }
                }
                let updated_session = serde_json::to_string(&session_user).unwrap();
                redis.set(&redis_session_key, &updated_session).unwrap();
            }
            Err(error) => {}
        }

        // 从会话管理器中移除会话
        let session_manager = self.state.session_manager.clone();
        let session_id = self.session_id.clone();

        tokio::spawn(async move {
            session_manager.remove_session(&session_id).await;
        });

        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                debug!("Received message: {}", text);
                // 尝试解析为WsContext
                if let Ok(ws_context) = serde_json::from_str::<WsContext>(&text) {
                    debug!("Received message: {:?}", ws_context);

                    // 检查是否是认证失败的消息
                    if ws_context.code == 401 {
                        if let Some(ref message) = ws_context.message {
                            warn!("Authentication failed: {}", message);
                            ctx.close(None); // 关闭连接
                            return;
                        }
                        ctx.close(None); // 关闭连接
                        return;
                    }

                    // 获取 sessionId
                    debug!("My session id = {}", self.session_id);

                    // 根据消息中的client_id发送给指定用户
                    if let Some(target_client_id) = ws_context.client_id {
                        // 在实际应用中，这里应该查找目标用户的连接并发送消息
                        debug!("Sending message to specific client: {}", target_client_id);
                        // 暂时将消息发回给自己，实际应用中应发送给target_client_id对应的客户端
                        // ctx.text(text);
                        let manager = self.state.session_manager.clone();
                        let msg = text.clone();

                        spawn(async move {
                            if let Some(addr) = manager.get_session(&*target_client_id).await {
                                addr.do_send(ServerText(msg.parse().unwrap()));
                            }
                        });
                    } else {
                        // 如果没有指定接收者，则将消息广播给所有连接的客户端
                        debug!("Broadcasting message to all clients");
                        ctx.text(text);
                    }
                } else {
                    // 无法解析为WsContext的消息，直接广播
                    debug!("Received non-WsContext message: {}", text);
                    ctx.text(text);
                }
            }
            Ok(ws::Message::Close(_)) => {
                info!("Client {} closed connection", self.session_id);
                ctx.stop();
            }
            _ => {
                // 处理其他类型的消息
                ctx.stop();
            }
        }
    }
}
