use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{
    Router,
    routing::{get, post},
};
use futures::StreamExt;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{Sender, channel};
use tokio::{pin, select};
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

    // TODO: research the influence of the buffer size
    let (tx, mut rc) = channel::<String>(10);
    let tx = Arc::new(tx);
    let mut queue = VecDeque::new();

    let app = Router::new()
        .route("/about", get(|| async { "this is an experiment" }))
        .route("/{user_id}", post(register_video))
        .layer(TraceLayer::new_for_http())
        .with_state(tx);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(listener, app).into_future();

    pin!(server);
    loop {
        select! {
            Some(job) = rc.recv() => {
                tracing::trace!("adding job to the queue: {:?}", job);
                queue.push_front(job);
            },
            _ = &mut server => {},
        }
    }
}

#[instrument]
async fn register_video(
    State(tx): State<Arc<Sender<String>>>,
    Path(id): Path<String>,
    body: Body,
) -> Result<StatusCode, StatusCode> {
    let mut video = body.into_data_stream();
    let path = format!("tmp/test_file_{}", id);
    let mut file = File::create(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(frame) = video.next().await {
        let chunk = frame.map_err(|_| StatusCode::BAD_REQUEST)?;
        file.write_all(&chunk)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.send(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
