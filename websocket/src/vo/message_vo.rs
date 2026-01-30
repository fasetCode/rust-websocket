use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct MessageVO {
    pub app_id: String,// app_id
    pub app_token: String,// app_token
    pub message: String,
    pub user_ids: Vec<String>,
}

// 节点消息转发
#[derive(Debug, Serialize, FromRow, Deserialize,Clone)]
pub struct NodeMessageVO {
    pub app_id: String,
    pub app_token: String,
    pub message: String,
    pub node_to: Vec<NodeToVo>,
}

#[derive(Debug, Serialize, FromRow, Deserialize,Clone)]
pub struct NodeToVo{
    pub base_url:String,
    pub user_ids: Vec<String>,
    // ip
    pub ip:String,
    // 端口
    pub port:u16,
}

#[derive(Debug, Serialize, FromRow, Deserialize,Clone)]
pub struct NodeTo{
    pub node: NodeToVo,
    pub app_id: String,
    pub app_token: String,
    pub message: String,
}

impl NodeToVo {
    pub fn new(base_url:String, user_ids: Vec<String>, ip:String, port:u16) -> Self {
        NodeToVo {
            base_url,
            user_ids,
            ip,
            port,
        }
    }
}


impl NodeMessageVO {
    pub fn init(app_id: String, app_token: String, message: String ) -> Self {
        NodeMessageVO {
            app_id,
            app_token,
            message,
            node_to: vec![],
        }
    }
}

