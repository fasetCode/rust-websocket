pub mod message_controller;
pub mod user_controller;

use actix_web::web;

// 定义一个函数来配置所有控制器路由
pub fn config_services(cfg: &mut web::ServiceConfig) {
    cfg
        // 用户控制器
        .service(user_controller::get_page)
        .service(user_controller::create_user)
        .service(user_controller::update_user)
        .service(user_controller::delete_user)
        .service(user_controller::login)

        // 消息转发
        .service(message_controller::node_push_handler)

        // 消息控制器
        .service(message_controller::message_push_handler)
        .service(message_controller::push_handler);
}
