mod controller;
mod web_socket;
mod db;
mod domain;
mod dao;
mod service;
mod common;
mod vo;
mod utils;
mod config;
mod props;
mod http;

use crate::web_socket::web_socket_server::{AppState, SessionManager, ws_handler};
use actix_web::{
    App, HttpServer,
    web::{self, Data},
};
use sqlx::postgres::PgPoolOptions;
use crate::config::middleware::AuthMiddleware;
use crate::config::redis_manager::RedisManager;
use crate::db::obj::DbState;
use crate::props::config::get_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("TODO: panic message");
    dotenvy::dotenv().ok();

    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.db_url)
        .await
        .expect("db connect failed");


    let db_state = Data::new(DbState { db:db.clone() });

    let redis_ws = Data::new(RedisManager::new(&config.redis_ws)
        .expect("redis connect failed"));

    let session_manager = SessionManager::new();
    let state = Data::new(AppState {
        app_name: "ws-gateway".into(),
        session_manager,
        redis: redis_ws,
        db: db.clone()
    });

    let redis = Data::new(RedisManager::new(&config.redis_url)
        .expect("redis connect failed"));


    HttpServer::new(move || {
        App::new()
            .wrap(AuthMiddleware)
            .app_data(state.clone())
            .app_data(db_state.clone())
            .app_data(redis.clone())
            .route("/ws", web::get().to(ws_handler))
            .configure(controller::config_services)
    })
    .bind(("0.0.0.0", config.port))?
    // 启动服务器并等待
    .run()
    .await
}