#!/usr/bin/env bash
# MotionArcade Gemini Pair Programmer Script
# Invokes gemini-cli with MotionArcade system engineering standards

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
REPO_ROOT=$(dirname "$SCRIPT_DIR")

SYSTEM_PROMPT="$REPO_ROOT/prompts/SYSTEM_PROMPT.md"

if ! command -v gemini &> /dev/null; then
    echo "Warning: 'gemini' CLI is not found in PATH."
    echo "Usage: ./scripts/gemini_pair.sh \"Task description or prompt\""
    exit 1
fi

echo "=== MotionArcade Gemini Pair Session ==="
echo "Repo Root: $REPO_ROOT"
echo "System Prompt: $SYSTEM_PROMPT"

cd "$REPO_ROOT"

if [ $# -eq 0 ]; then
    gemini --system-instruction "$SYSTEM_PROMPT"
else
    gemini --system-instruction "$SYSTEM_PROMPT" "$@"
fi

