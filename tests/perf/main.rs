use std::error::Error;
use tokio::fs::File;
use tokio::task::JoinSet;
use tokio_util::io::ReaderStream;

#[tokio::test]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut join_set = JoinSet::new();
    for i in 0..100 {
        let url = format!("http://localhost:3000/{}", i);
        join_set.spawn(async move {
            let client = reqwest::Client::new();
            let file = File::open("tests/data/trial_video.mp4").await.unwrap();
            let stream = ReaderStream::new(file);
            let body = reqwest::Body::wrap_stream(stream);
            client.post(url).body(body).send().await
        });
    }

    join_set.join_all().await;

    Ok(())
}
