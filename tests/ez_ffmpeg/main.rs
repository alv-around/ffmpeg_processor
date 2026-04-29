use ez_ffmpeg::FfmpegContext;
use ez_ffmpeg::FfmpegScheduler;
use same_file::is_same_file;

#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build the FFmpeg context
    let context = FfmpegContext::builder()
        .input("tests/data/trial_video.mp4")
        .output("tmp/test_output.mp4")
        .build()?;

    // 2. Run it via FfmpegScheduler (synchronous mode)
    let result = FfmpegScheduler::new(context).start()?.wait();
    result?; // Propagate any errors that occur

    is_same_file("tmp/test_output.mp4", "tests/data/output.mp4")?;

    Ok(())
}
