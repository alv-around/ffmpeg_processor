use std::env;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::fs::File;
use tokio::task::JoinSet;
use tokio_util::io::ReaderStream;

#[ignore]
#[tokio::test]
async fn main() -> Result<(), Box<dyn Error>> {
    let n = env::var("N").map_or(1000usize, |x| x.parse().expect("N should be a number"));
    let client_timeout = env::var("CLIENT_TIMEOUT").map_or(20u64, |x| {
        x.parse().expect("CLIENT_TIMEOUT should be a number")
    });
    let min_success_rate = env::var("SUCCESS_RATE").map_or(9900, |x| {
        x.parse()
            .expect("SUCCESS_RATE should be a number (percent * 100)")
    });

    let error_cnt = Arc::new(AtomicUsize::new(0));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(client_timeout))
        .build()?;

    let mut join_set = JoinSet::new();

    for i in 0..n {
        let client = client.clone();
        let url = format!("http://localhost:3000/{}", i);
        let errors = error_cnt.clone();
        join_set.spawn(async move {
            let file = File::open("tests/data/trial_video.mp4").await.unwrap();
            let stream = ReaderStream::new(file);
            let body = reqwest::Body::wrap_stream(stream);

            match client.post(&url).body(body).send().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Request {} failed: {}", i, e);
                    errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
    }

    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            eprintln!("A spawned task panicked: {}", e);
        }
    }

    let error_cnt: usize = error_cnt.load(Ordering::SeqCst);
    let success_rate: usize = (n - error_cnt).div_ceil(n) * 100 * 100;
    println!(
        "Erroneous requests: {:?} Total Success Rate: {:?}",
        error_cnt, success_rate
    );

    if success_rate < min_success_rate {
        return Err("Test did not reach minimum success rate".into());
    }
    Ok(())
}
