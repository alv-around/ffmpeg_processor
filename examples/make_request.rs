#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let handler = tokio::spawn(async move {
        let url = &args[1];
        match reqwest::get(url).await {
            Ok(res) => {
                let body = res.text().await.unwrap();
                println!("request response:\n{:?}", body);
            }
            Err(_) => println!("error in request"),
        }
    });
    handler.await.unwrap();
}
