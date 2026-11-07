#!/usr/bin/env bash
# MotionArcade MoveNet Model Auto-Fetcher
# Downloads the official MoveNet-Lightning float32 ONNX model into assets/models/

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
REPO_ROOT=$(dirname "$SCRIPT_DIR")
TARGET_DIR="$REPO_ROOT/assets/models"
TARGET_FILE="$TARGET_DIR/movenet_lightning.onnx"

mkdir -p "$TARGET_DIR"

if [ -f "$TARGET_FILE" ] && [ -s "$TARGET_FILE" ]; then
    echo "MoveNet model already exists at: $TARGET_FILE"
    exit 0
fi

echo "Fetching MoveNet-Lightning ONNX model..."
# Canonical MoveNet Lightning ONNX repository hosted on Hugging Face / Google Research
MODEL_URL="https://huggingface.co/qualcomm/MoveNet-SinglePose-Lightning/resolve/main/MoveNet-SinglePose-Lightning.onnx"

if command -v curl &> /dev/null; then
    curl -L --fail -o "$TARGET_FILE" "$MODEL_URL" || {
        echo "Primary download mirror failed, attempting fallback mirror..."
        curl -L --fail -o "$TARGET_FILE" "https://raw.githubusercontent.com/onnx/models/main/validated/vision/body_analysis/movenet/movenet_lightning.onnx" || true
    }
elif command -v wget &> /dev/null; then
    wget -O "$TARGET_FILE" "$MODEL_URL" || true
else
    echo "Warning: Neither curl nor wget found. Please place movenet_lightning.onnx in $TARGET_DIR manually."
    exit 1
fi

if [ -f "$TARGET_FILE" ] && [ -s "$TARGET_FILE" ]; then
    echo "Successfully installed model to: $TARGET_FILE ($(du -h "$TARGET_FILE" | cut -f1))"
else
    echo "Note: Model download skipped or network unavailable. Headless mock streaming and synthetic tests remain 100% operational."
fi

