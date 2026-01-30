use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Transform},
    Error, web
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::rc::Rc;
use actix_web::web::put;
use crate::config::redis_manager::RedisManager;
use crate::props::config::get_config;

/// 自定义中间件
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        let ignore_paths = vec!["/ws","/api/login","/api/node/push"];

        Box::pin(async move {
            let path = req.path();

            if path == "/api/node/push" {
                let config = get_config().unwrap();

                if let Some(loc_to_token_header) = req.headers().get("loc_to_token") {
                    let loc_to_token = loc_to_token_header.to_str().unwrap_or("").to_string();
                    if config.node_token != loc_to_token {
                        return Err(actix_web::error::ErrorUnauthorized("Invalid node token"));
                    }
                } else {
                    return Err(actix_web::error::ErrorUnauthorized("Missing node token"));
                }
            }

            // 检查是否在忽略列表中
            let should_ignore = ignore_paths.iter().any(|ignore_path| {
                if ignore_path.ends_with('/') {
                    path.starts_with(ignore_path)  // 前缀匹配，如 /static/
                } else {
                    path == *ignore_path  // 精确匹配
                }
            });

            if should_ignore {
                // 放行，不检查token
                return svc.call(req).await;
            }

            // ✅ 拦截逻辑
            if let Some(auth_header) = req.headers().get("Authorization") {
                let token = auth_header.to_str().unwrap_or("").trim_start_matches("Bearer ").to_string();

                // 从请求中获取app数据（包括Redis连接）
                if let Some(app_data) = req.app_data::<web::Data<RedisManager>>() {
                    let redis_manager = app_data.as_ref();

                    // redis 查询是否有token
                    match redis_manager.get(&*("user_token:".to_owned() + &token)) {
                        Ok(_) => {
                            // token 存在，继续请求
                            svc.call(req).await
                        },
                        Err(_) => {
                            // token 不存在，返回 401
                            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                        }
                    }
                } else {
                    // Redis连接不可用，返回错误
                    Err(actix_web::error::ErrorInternalServerError("Redis connection not available"))
                }
            } else {
                // 没有 token
                Err(actix_web::error::ErrorUnauthorized("Missing token"))
            }
        })
    }
}