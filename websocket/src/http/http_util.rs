use reqwest::Client;
use serde_json::Value;

/// 发送 HTTP POST 请求
/// - url: 请求地址
/// - data: JSON 字符串
/// - headers: 可选 HTTP 头部
/// 返回 Result<Value, String>，请求失败或解析失败都会返回 Err
pub async fn http_post(
    url: &str,
    data: &str,
    headers: &[(&str, &str)],
) -> Result<Value, String> {
    let client = Client::new();

    // 构建请求
    let mut request = client
        .post(url)
        .body(data.to_string())
        .header("Content-Type", "application/json");

    for (k, v) in headers {
        request = request.header(*k, *v);
    }

    // 发送请求
    let resp = match request.send().await {
        Ok(r) => {
            println!("请求成功发送到: {}", url);
            r
        }
        Err(e) => {
            println!("请求失败啦！Error: {}", e);
            return Err(format!("Request failed: {}", e));
        }
    };

    // 检查 HTTP 状态
    if resp.status().is_success() {
        println!("HTTP 状态成功: {}", resp.status());
        match resp.json::<Value>().await {
            Ok(json) => {
                println!("解析 JSON 成功: {}", json);
                Ok(json)
            }
            Err(e) => {
                println!("解析 JSON 失败: {}", e);
                Err(format!("Parse JSON failed: {}", e))
            }
        }
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        println!("HTTP 请求返回错误状态 {}: {}", status, text);
        Err(format!("HTTP {}: {}", status, text))
    }
}
