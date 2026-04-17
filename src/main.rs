use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{
    Router,
    routing::{get, post},
};
use ez_ffmpeg::{FfmpegContext, FfmpegScheduler};
use futures::StreamExt;
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

    let app = Router::new()
        .route("/about", get(|| async { "this is an experiment" }))
        .route("/{user_id}", post(register_video))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    let _server = axum::serve(listener, app).await;
}

#[instrument]
async fn register_video(Path(id): Path<String>, body: Body) -> Result<StatusCode, &'static str> {
    let mut video = body.into_data_stream();
    let path = format!("tmp/test_file_{}.mp4", id);
    let output = format!("tmp/output_{}.mp4", id);
    let mut file = File::create(&path)
        .await
        .map_err(|_| "error creating file")?;

    while let Some(frame) = video.next().await {
        let chunk = frame.map_err(|_| "error reading file")?;
        file.write_all(&chunk)
            .await
            .map_err(|_| "error backing file")?;
    }

    tracing::debug!("file stored in path: {}", path);

    // 1. Build the FFmpeg context
    let context = FfmpegContext::builder()
        .input(path)
        .output(output)
        .build()
        .map_err(|_| "error initializing ffmpeg")?;

    let scheduler = FfmpegScheduler::new(context)
        .start()
        .map_err(|_| "error processing video")?
        .await
        .map_err(|_| "error processing video")?;

    Ok(StatusCode::OK)
}
