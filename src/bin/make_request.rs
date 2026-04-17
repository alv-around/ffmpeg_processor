use std::error::Error;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("test/trial_video.mp4").await?;
    let stream = ReaderStream::new(file);
    let body = reqwest::Body::wrap_stream(stream);

    let client = reqwest::Client::new();
    let handler = tokio::spawn(async move {
        match client.post("0.0.0.0:3000/11").body(body).send().await {
            Ok(res) => {
                let body = res.text().await.unwrap();
                println!("request response:\n{:?}", body);
            }
            Err(_) => println!("error in request"),
        }
    });
    handler.await.unwrap();

    Ok(())
}
