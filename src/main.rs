use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{
    Router,
    routing::{get, post},
};
use futures::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/about", get(|| async { "this is an experiment" }))
        .route("/{user_id}", post(register_video));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on 0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn register_video(Path(id): Path<String>, body: Body) -> Result<StatusCode, StatusCode> {
    let mut video = body.into_data_stream();
    let mut file = File::create(format!("tmp/test_file_{}", id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    while let Some(frame) = video.next().await {
        let chunk = frame.map_err(|_| StatusCode::BAD_REQUEST)?;
        file.write_all(&chunk)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(StatusCode::OK)
}
