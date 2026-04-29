use std::error::Error;
use std::time::Duration;
use tokio::fs::File;
use tokio::task::JoinSet;
use tokio_util::io::ReaderStream;

#[tokio::test]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Add atomic counts to see how many request are dropped
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let mut join_set = JoinSet::new();

    for i in 0..1000 {
        let client = client.clone();
        let url = format!("http://localhost:3000/{}", i);
        join_set.spawn(async move {
            let file = File::open("tests/data/trial_video.mp4").await.unwrap();
            let stream = ReaderStream::new(file);
            let body = reqwest::Body::wrap_stream(stream);

            match client.post(&url).body(body).send().await {
                Ok(res) => {
                    println!("Request {} finished: {}", i, res.status());
                }
                Err(e) => {
                    eprintln!("Request {} failed: {}", i, e);
                }
            }
        });
    }

    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            eprintln!("A spawned task panicked: {}", e);
        }
    }

    Ok(())
}
