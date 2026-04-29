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

#[cfg(test)]
mod tests {
    use super::*;
    use same_file::is_same_file;

    #[tokio::test]
    async fn test_ffmpeg_wrapper() {
        // 1. Build the FFmpeg context
        run_ffmpeg(
            "tests/data/trial_video.mp4".to_string(),
            "tmp/test_output.mp4".to_string(),
        )
        .await
        .unwrap();

        assert!(is_same_file("tmp/test_output.mp4", "tests/data/output.mp4").is_ok());
    }
}
