use actix::spawn;
use crate::web_socket::web_socket_server::{AppState, PushRequest, ServerText, SessionManager};
use actix_web::{
    HttpResponse, post,
    web::{self, Data},
};
use crate::http::http_util::http_post;
use crate::props::config::{get_config, Config};
use crate::vo::message_vo::{MessageVO, NodeMessageVO, NodeTo, NodeToVo};
use crate::web_socket::app_node::SessionUser;

#[post("/api/push")]
pub async fn push_handler(body: web::Json<PushRequest>, state: Data<AppState>) -> HttpResponse {
    let manager = state.session_manager.clone();
    let _ = &match body.client_id.as_deref() {
        Some(x) => {
            let addr = manager.get_session(&x);
            if let Some(addr) = addr.await {
                addr.do_send(ServerText(body.data.clone().unwrap()));
            }
        }
        None => todo!(),
    };
    HttpResponse::Ok().body(format!("push via {}", state.app_name))
}

#[post("/api/message/push")]
pub async fn message_push_handler(body: web::Json<MessageVO>, state: Data<AppState>) -> HttpResponse {
    let config = get_config().expect("TODO: panic message");

    let node_config = config.clone();
    // 获取
    let redis = state.redis.clone();
    let manager = state.session_manager.clone();
    let mut node_list :NodeMessageVO = NodeMessageVO::init(
        body.app_id.clone(),
        body.app_token.clone(),
        body.message.clone(),
    );
    for user_id in &body.user_ids {
        let redis_session_key = format!(
            "web:socket:app_id:{}:user:id:{}",
            body.app_id, user_id
        );

        let user_session =
            redis.get_not_null(&redis_session_key)
                .expect("TODO: panic message");

        if user_session.is_empty() {
            continue;
        }

        let mut is_update = false;
        let mut session_user: SessionUser = serde_json::from_str(&user_session).unwrap();
        let mut nodes_to_remove = Vec::new();
        for (index, user) in session_user.nodes.iter().enumerate() {
            let url = format!("http://{}:{}/api/node/push", user.ip, user.port);
            if user.ip == config.app_ip && config.port == user.port { // 链接在这个链接上发送数据
                let bool = send_message(&body.message, &user.session_id,&manager);
                if !bool.await { // 节点不存在
                    // 记录要删除的节点索引
                    nodes_to_remove.push(index);
                    is_update=true;
                }
            }else{
                let mut is_add = true;
                // 不在这个链接上发送数据
                for node in node_list.node_to.iter() {
                    if url == node.base_url{
                        for userid in node.user_ids.iter() {
                            if userid == user_id {
                                is_add = false;
                            }
                        }
                    }
                }
                if is_add {
                    is_add = true;
                    let mut found = false;
                    for node_mut in node_list.node_to.iter_mut() {
                        if url == node_mut.base_url {
                            node_mut.user_ids.push(user_id.clone());
                            is_add = false;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        let add_node = NodeToVo::new(
                            url.clone(),
                            vec![user_id.clone()],
                            user.ip.clone(),
                            user.port
                        );
                        node_list.node_to.push(add_node);
                    }
                }
            }
        }



        // 逆序删除节点，避免索引变化问题
        for &index in nodes_to_remove.iter().rev() {
            session_user.nodes.remove(index);
        }
        if is_update {
            let data = serde_json::to_string(&session_user).unwrap();
            redis.set(&redis_session_key, &data).expect("TODO: panic message");
        }
    }


    // 节点转发
    spawn(async move {
        forward_nodes(node_list, node_config, body).await;
    });

    HttpResponse::Ok().body(format!("push via {}", "aaa"))
}



// 节点转发 的 消息
#[post("/api/node/push")]
pub async fn node_push_handler(body: web::Json<NodeTo>, state: Data<AppState>) -> HttpResponse {

    let redis = state.redis.clone();
    let manager = state.session_manager.clone();

    let config = get_config().unwrap();

    for user_id in body.node.user_ids.iter()  {
        let redis_session_key = format!(
            "web:socket:app_id:{}:user:id:{}",
            body.app_id, user_id
        );

        let user_session =
            redis.get_not_null(&redis_session_key)
                .expect("TODO: panic message");

        let mut session_user: SessionUser = serde_json::from_str(&user_session).unwrap();
        let mut nodes_to_remove = Vec::new();
        let mut is_update = false;
        for (index, user) in session_user.nodes.iter().enumerate() {
            if user.ip == config.app_ip && config.port == user.port { // 链接在这个链接上发送数据
                let bool = send_message(&body.message, &user.session_id,&manager);
                if !bool.await { // 节点不存在
                    // 记录要删除的节点索引
                    nodes_to_remove.push(index);
                    is_update=true;
                }
            }
        }
        // 逆序删除节点，避免索引变化问题
        for &index in nodes_to_remove.iter().rev() {
            session_user.nodes.remove(index);
        }
        if is_update {
            let data = serde_json::to_string(&session_user).unwrap();
            redis.set(&redis_session_key, &data).expect("TODO: panic message");
        }
    }
    HttpResponse::Ok().body(format!("push via {}", "aaa"))
}


async fn send_message(message: &String, session_id: &String, manager: &SessionManager) -> bool {
    let addr = manager.get_session(session_id).await;
    
    if let Some(addr) = addr {
        addr.do_send(ServerText(message.clone()));
        true
    } else {
        false
    }
}


// 节点转发
async fn forward_nodes(
    node_list: NodeMessageVO,
    node_config: Config,
    body: web::Json<MessageVO>
) {

    for node in node_list.node_to.iter() {


        let mut headers: Vec<(&str, &str)> = vec![];

        if let Some(nodes) = &node_config.node_config {
            for node_cfg in nodes {
                if node_cfg.port == node.port && node_cfg.ip == node.ip {
                    headers.push(("loc_to_token", &node_cfg.token));
                }
            }
        }
        headers.push(("Content-Type", "application/json"));

        let data: NodeTo = NodeTo{
            node: node.clone(),
            app_id: body.app_id.clone(),
            app_token: body.app_token.clone(),
            message: body.message.clone(),
        };
        let data = &serde_json::to_string(&data).unwrap();
        match http_post(&node.base_url,data,&headers).await {
            Ok(_) => {

            },
            Err(_) => {

            }
        }
    }
}
