#!/bin/bash
set -e

PI_HOST="pidisplay.local"
PI_PATH="/home/kivlor/pi-display"
BINARY="target/aarch64-unknown-linux-gnu/release/pi-display"

echo "Stopping service..."
ssh "$PI_HOST" "sudo systemctl stop pi-display"

echo "Deploying to $PI_HOST..."
scp "$BINARY" "$PI_HOST:$PI_PATH/"

echo "Starting service..."
ssh "$PI_HOST" "sudo systemctl start pi-display"

echo "Done."
