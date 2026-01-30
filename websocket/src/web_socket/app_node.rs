use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize,Serialize)]
pub struct AppNode {
    pub ip: String,
    pub port: u16,
    pub session_id: String
}

#[derive(Debug, Deserialize,Serialize)]
pub struct SessionUser {
    pub nodes: Vec<AppNode>,
}


impl AppNode {
    pub fn new(ip: String, port: u16, session_id: String) -> AppNode {
        AppNode {
            ip,
            port,
            session_id
        }
    }
}


impl SessionUser {
    
    pub fn new(nodes: Vec<AppNode>) -> SessionUser {
        SessionUser {
            nodes
        }
    }
    
}