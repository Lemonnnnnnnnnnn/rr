use rt_1::{HttpClient, ProxyConfig};

#[tokio::test]
async fn test_get_httpbin() {
    let mut client = HttpClient::new();
    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await
        .expect("请求失败");

    println!("状态码: {}", response.status_code);
    println!("响应内容: {}", response.body);
    assert!(response.is_success());
}

// 测试代理
#[tokio::test]
async fn test_proxy() {
    let mut client = HttpClient::with_proxy(ProxyConfig::http("127.0.0.1", 7890));
    let response = client
        .get("https://e-hentai.org")
        .send()
        .await
        .expect("请求失败");
    assert!(response.is_success());
    println!("{}", response.body);
}

// #[test]
// fn test_get_18comic() {
//     let client = HttpClient::with_proxy().expect("创建客户端失败");
//     let response = client.get("https://18comic.ink/photo/292986").expect("请求失败");
//     assert!(response.is_success());
// }

// 测试并行请求
#[tokio::test]
async fn test_concurrent_requests() {
    let mut handles = vec![];

    for _ in 0..10 {
        let handle = tokio::spawn(async {
            let mut client = HttpClient::new();
            let response = client
                .get("https://httpbin.org/get")
                .send()
                .await
                .expect("请求失败");
            
            println!("状态码: {}", response.status_code);
            println!("响应内容: {}", response.body);
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("所有请求完成");
}
