use actix_web::{get, post,put,delete, web, HttpRequest, HttpResponse};
use actix_web::web::Data;
use serde_json::json;
use crate::common::dto::ResultVo;
use crate::service::user_service;
use crate::domain::user::{UserCreate, UserLogin, UserQuery, UserUpdate};
use crate::db::obj::DbState;

#[get("/api/user/page")]
pub async fn get_page(req: HttpRequest, query: web::Query<UserQuery>) -> HttpResponse {
    // 使用从应用状态获取的数据库连接池
    let db_state = req.app_data::<Data<DbState>>().unwrap();
    let pool = &db_state.db;

    match user_service::get_page(pool, query.into_inner()).await {
        Ok(users) => {
            let response = json!(ResultVo::ok_with(users));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!(
                ResultVo::<()>::error(1, e.to_string())
            ))
        }
    }
}


#[post("/api/user")]
pub async fn create_user(user_create: web::Json<UserCreate>,req: HttpRequest) -> HttpResponse {

    let db_state = req.app_data::<Data<DbState>>().unwrap();
    let pool = &db_state.db;

    match user_service::create_user(pool, user_create.into_inner()).await {
        Ok(user) => {
            let response = json!(ResultVo::ok_with(user));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!(
                ResultVo::<()>::error(1, e.to_string())
            ))
        }
    }
}

#[put("/api/user")]
pub async fn update_user(user_update: web::Json<UserUpdate>,req: HttpRequest) -> HttpResponse {
    let db_state = req.app_data::<Data<DbState>>().unwrap();
    let pool = &db_state.db;
    match user_service::update_user(pool, user_update.into_inner()).await {
        Ok(number) => {
            let response = json!(ResultVo::ok_with(number));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!(
                ResultVo::<()>::error(1, e.to_string())
            ))
        }
    }
}

#[delete("/api/user/{id}")]
pub async fn delete_user(req: HttpRequest) -> HttpResponse {
    let user_id: i32 = req.match_info().get("id").unwrap().parse().unwrap();
    let db_state = req.app_data::<Data<DbState>>().unwrap();
    let pool = &db_state.db;

    match user_service::delete_user(pool, user_id).await {
        Ok(number) => {
            let response = json!(ResultVo::ok_with(number));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!(
                ResultVo::<()>::error(1, e.to_string())
            ))
        }
    }
}

#[post("/api/login")]
pub async fn login(user_create: web::Json<UserLogin>,req: HttpRequest) -> HttpResponse {
    let db_state = req.app_data::<Data<DbState>>().unwrap();
    let pool = &db_state.db;
    let redis_manager = req.app_data::<Data<crate::config::redis_manager::RedisManager>>().unwrap();

    match user_service::login(pool, user_create.into_inner(), redis_manager).await {
        Ok(token) => {
            let response = json!(ResultVo::ok_with(token));
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(json!(
                ResultVo::<()>::error(1, e.to_string())
            ))
        }
    }
}