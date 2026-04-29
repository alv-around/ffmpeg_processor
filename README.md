# FFMPEG Worker

This server provides a http server to process videos with ffmpeg. It is
equivalent to running manually:

```console
ffmpeg -i tests/data/trial_video.mp4 tests/data/output.mp4
```

## Run

```console
nix develop
cargo run
```

## Test

To run the tests:

```console
cargo test
cargo bench
# To run the stress test
cargo test -- --features stress-test
```

> to run the stress test, you need to have an instance of ffmpeg_processor
> running
