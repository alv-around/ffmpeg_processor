#!/bin/bash

for i in {1..5}; do
        curl -X POST --data-binary @"$(pwd)"/test/trial_video.mp4 http://0.0.0.0:3000/"${i}"
done
