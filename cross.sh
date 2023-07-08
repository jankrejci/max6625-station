#!/bin/sh

docker run \
    -it \
	--network="host" \
    -e RUST_LOG="$RUST_LOG" \
    -v ".:/project" \
	-v "cargo-dir:/home/pi/.cargo" \
    -v "${SSH_AUTH_SOCK}:/ssh-agent" \
	rpi-cross-compile-image \
    $@
