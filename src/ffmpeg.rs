use ez_ffmpeg::{FfmpegContext, FfmpegScheduler};

pub async fn run_ffmpeg(input_path: String, output_path: String) -> Result<(), &'static str> {
    // 1. Build the FFmpeg context
    let context = FfmpegContext::builder()
        .input(input_path)
        .output(output_path)
        .build()
        .map_err(|_| "error initializing ffmpeg")?;

    FfmpegScheduler::new(context)
        .start()
        .map_err(|_| "error processing video")?
        .await
        .map_err(|_| "error processing video")?;

    Ok(())
}
