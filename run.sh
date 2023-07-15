#!/bin/sh

BIN_PATH="$1"
BIN_FOLDER=$(dirname "$BIN_PATH")
BIN_FILE=$(basename "$BIN_PATH")
TEMP_FOLDER="/tmp"

TARGET="$2"
TARGET_FOLDER="/tmp"

# Run the binary if the runner is executed locally
if [ -z "$TARGET" ]; then
    ./$BIN_PATH
    exit;
fi
    
# Compress the binary
tar -czvf "$TEMP_FOLDER/$BIN_FILE.tar.gz"  -C "$BIN_FOLDER" "$BIN_FILE"

# Copy the compressed binary to the remote host
scp \
    -o "ForwardAgent yes" \
    -o "StrictHostKeyChecking=no" \
    "$TEMP_FOLDER/$BIN_FILE.tar.gz" \
    "$TARGET:$TARGET_FOLDER"

scp -o "ForwardAgent yes" "config.toml" "$TARGET:$TARGET_FOLDER"
scp -o "ForwardAgent yes" "Rocket.toml" "$TARGET:$TARGET_FOLDER"


# Decompress the binary
ssh -A "$TARGET" "tar -xzvf \"$TARGET_FOLDER/$BIN_FILE.tar.gz\"" -C "$TARGET_FOLDER"

ssh -A "$TARGET" "killall $BIN_FILE"

if [ -z "$RUST_LOG" ]; then
    RUST_LOG="info,rocket=warn";
fi

# Run the binary on the remote host via SSH
ssh -A -t "$TARGET" "cd $TARGET_FOLDER; RUST_LOG=$RUST_LOG ./$BIN_FILE"
