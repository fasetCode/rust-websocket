use std::{env, fs, sync::Arc};
use std::path::Path;
use std::sync::{Mutex};
use std::time::{Duration, Instant};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub port: u16,
    pub redis_url: String,
    pub redis_ws: String,
    pub db_max_connections: u32,
    // token过期时间
    pub token_ex: u64,
    // 本地缓存控制参数
    #[serde(default = "default_true")]
    pub local_cache_enabled: bool,
    #[serde(default = "default_cache_ttl")]
    pub local_cache_ttl: u64,  // TTL in seconds

    pub node_token: String,

    // 设置 应用的IP
    pub app_ip: String,

    // 节点通信权限
    pub node_config: Option<Vec<NodeConfig>>,

}
#[derive(Deserialize, Debug, Clone)]
pub struct NodeConfig{
    // ip
    pub ip: String,
    // 端口
    pub port: u16,
    // 权限标识
    pub token: String,
}


// 默认值函数
fn default_true() -> bool { true }
fn default_cache_ttl() -> u64 { 300 } // 5 minutes default
fn default_cache_size_limit() -> usize { 100 }

// 全局缓存
static CONFIG_CACHE: LazyLock<Arc<Mutex<Option<(Config, Instant)>>>> = 
    LazyLock::new(|| Arc::new(Mutex::new(None)));

fn get_cache() -> Arc<Mutex<Option<(Config, Instant)>>> {
    CONFIG_CACHE.clone()
}

// 同步版本
pub fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    // 尝试从缓存获取
    let cache = get_cache(); // 先获取 Arc

    let mut cache_guard = cache.lock().unwrap();
    if let Some((ref cached_config, timestamp)) = *cache_guard {
        // 检查是否过期
        if cached_config.local_cache_enabled &&
            timestamp.elapsed() < Duration::from_secs(cached_config.local_cache_ttl) {
            println!("✅ Returning config from cache");
            return Ok(cached_config.clone());
        }
    }

    drop(cache_guard); // 释放锁以便重新加载

    // 缓存未命中或已过期，从文件加载
    let possible_paths = vec![
        "./config.yaml".to_string(),
        env::var("CARGO_MANIFEST_DIR")
            .map(|dir| format!("{}/config.yaml", dir))
            .unwrap_or_default(),
        get_exe_dir()
            .map(|dir| format!("{}/config.yaml", dir))
            .unwrap_or_default(),
    ];

    // 使用引用迭代，不消耗所有权
    for path in &possible_paths {
        if Path::new(path).exists() {
            println!("✅ Loading config from: {}", path);
            let yaml_content = fs::read_to_string(path)?;
            let config: Config = serde_yaml::from_str(&yaml_content)?;
            println!("{:?}", config);

            // 存储到缓存
            if config.local_cache_enabled {
                let cache = get_cache(); // 重新获取 Arc
                let mut cache_guard = cache.lock().unwrap();
                *cache_guard = Some((config.clone(), Instant::now()));
            }

            return Ok(config);
        }
    }

    // 现在 possible_paths 仍然可用
    let error_msg = format!(
        "Config file not found. Tried:\n{}",
        possible_paths.join("\n")
    );
    Err(error_msg.into())
}


// 获取可执行文件所在目录
fn get_exe_dir() -> Option<String> {
    env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_string_lossy().to_string()))
}

// 更简单的版本：始终从项目根目录加载
// pub fn get_config_simple() -> Result<Config, Box<dyn std::error::Error>> {
//     // 获取项目根目录（Cargo.toml所在目录）
//     let project_root = env::var("CARGO_MANIFEST_DIR")
//         .unwrap_or_else(|_| ".".to_string());
//
//     let config_path = format!("{}/config.yaml", project_root);
//     println!("Looking for config at: {}", config_path);
//
//     let yaml_content = fs::read_to_string(&config_path)?;
//     let config: Config = serde_yaml::from_str(&yaml_content)?;
//     println!("✅ Config loaded: {:?}", config);
//
//     Ok(config)
// }


// 测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let config = get_config().expect("TODO: panic message");
        println!("{:?}", config);
    }
}