use ffmpeg_processor::ffmpeg::run_ffmpeg;
use ffmpeg_processor::tracing::init_tracing;

use axum::body::Body;
use axum::extract::Path;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{
    Router,
    routing::{get, post},
};
use futures::StreamExt;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tower_http::trace::TraceLayer;
use tracing::instrument;

#[tokio::main]
async fn main() {
    init_tracing();

    let app = Router::new()
        .route("/about", get(|| async { "this is an experiment" }))
        .route("/{video_id}", post(register_video))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    let _server = axum::serve(listener, app).await;
}

#[instrument]
async fn register_video(
    Path(id): Path<String>,
    body: Body,
) -> Result<impl IntoResponse, &'static str> {
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

    run_ffmpeg(path, output.clone()).await?;

    match fs::read(output).await {
        Ok(bytes) => Ok(([(header::CONTENT_TYPE, "video/mp4")], bytes)),
        Err(_) => Err("error reading the output file"),
    }
}
