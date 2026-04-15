use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{
    Router,
    routing::{get, post},
};
use ez_ffmpeg::{FfmpegContext, FfmpegScheduler};
use futures::StreamExt;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create the the workers queue
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let queue_thread = Arc::clone(&queue);
    thread::spawn(move || {
        loop {
            if let Ok(mut queue) = queue_thread.lock()
                && let Some(job) = queue.pop_back()
            {
                tracing::debug!(
                    "Substracting job to working queue. Queue length: {}",
                    queue.len()
                );
                tracing::info!("starting job: {}", job);
            }
        }
    });

    let app = Router::new()
        .route("/about", get(|| async { "this is an experiment" }))
        .route("/{user_id}", post(register_video))
        .layer(TraceLayer::new_for_http())
        .with_state(queue);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    let _server = axum::serve(listener, app).await;
}

#[instrument]
async fn register_video(
    State(queue): State<Arc<Mutex<VecDeque<String>>>>,
    Path(id): Path<String>,
    body: Body,
) -> Result<StatusCode, StatusCode> {
    let mut video = body.into_data_stream();
    let path = format!("tmp/test_file_{}.mp4", id);
    let output = format!("tmp/output_{}.mp4", id);
    let mut file = File::create(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(frame) = video.next().await {
        let chunk = frame.map_err(|_| StatusCode::BAD_REQUEST)?;
        file.write_all(&chunk)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // 1. Build the FFmpeg context
    let context = FfmpegContext::builder()
        .input(path)
        .output(output)
        .build()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let scheduler = FfmpegScheduler::new(context)
        .start()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // 3. Block until it's finished
    scheduler.wait().map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::OK)
}
