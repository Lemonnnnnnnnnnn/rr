use rt_1::{proxy::ProxyConfig, HttpClient};

#[test]
fn test_get_httpbin() {
    let client = HttpClient::new().expect("创建客户端失败");
    let response = client.get("https://httpbin.org/get").expect("请求失败");
    assert!(response.is_success());
}

#[test]
fn test_get_e_hentai() {
    let client = HttpClient::new().expect("创建客户端失败");
    let response = client.get("https://e-hentai.org").expect("请求失败");
    assert!(response.is_success());
}

// #[test]
// fn test_get_18comic() {
//     let client = HttpClient::with_proxy().expect("创建客户端失败");
//     let response = client.get("https://18comic.ink/photo/292986").expect("请求失败");
//     assert!(response.is_success());
// }

// 测试代理
#[test]
fn test_proxy() {
    let client =
        HttpClient::with_proxy(ProxyConfig::http("127.0.0.1", 7890)).expect("创建客户端失败");
    let response = client.get("https://httpbin.org/get").expect("请求失败");
    assert!(response.is_success());
}
