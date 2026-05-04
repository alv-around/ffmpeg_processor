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
```

### Stress Test

To run the stress test

```
$TEST_ENV_VARS cargo test --test harness -- --no-capture --ignored
```

> to run the stress test, you need to have an instance of ffmpeg_processor
> running
