#!/bin/bash

docker build . \
    --build-arg UID=$(id -u ${USER}) \
    --build-arg GID=$(id -g ${USER}) \
    -t rpi-cross-compile-image \
	"$@"
