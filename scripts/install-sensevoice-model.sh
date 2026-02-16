#!/usr/bin/env bash
set -euo pipefail

MODEL_ARCHIVE="sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2"
MODEL_DIR_NAME="sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17"
MODEL_URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/${MODEL_ARCHIVE}"
TARGET_ROOT="${HOME}/Library/Application Support/com.mascribe/models/sensevoice"
TARGET_DIR="${TARGET_ROOT}/${MODEL_DIR_NAME}"

echo "[model] Target: ${TARGET_DIR}"
mkdir -p "${TARGET_ROOT}"

if [[ -f "${TARGET_DIR}/model.int8.onnx" && -f "${TARGET_DIR}/tokens.txt" ]]; then
  echo "[model] Already installed. Nothing to do."
  exit 0
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARCHIVE_PATH="${TMP_DIR}/${MODEL_ARCHIVE}"

echo "[model] Downloading model archive..."
if command -v curl >/dev/null 2>&1; then
  curl -L --fail --retry 3 -o "${ARCHIVE_PATH}" "${MODEL_URL}"
elif command -v wget >/dev/null 2>&1; then
  wget -O "${ARCHIVE_PATH}" "${MODEL_URL}"
else
  echo "[model] ERROR: need curl or wget"
  exit 1
fi

echo "[model] Extracting archive..."
tar -xjf "${ARCHIVE_PATH}" -C "${TMP_DIR}"

if [[ ! -d "${TMP_DIR}/${MODEL_DIR_NAME}" ]]; then
  echo "[model] ERROR: extracted folder not found: ${MODEL_DIR_NAME}"
  exit 1
fi

mkdir -p "${TARGET_ROOT}"
rm -rf "${TARGET_DIR}"
mv "${TMP_DIR}/${MODEL_DIR_NAME}" "${TARGET_DIR}"

if [[ ! -f "${TARGET_DIR}/model.int8.onnx" || ! -f "${TARGET_DIR}/tokens.txt" ]]; then
  echo "[model] ERROR: model files are incomplete after install"
  exit 1
fi

echo "[model] Installed successfully."
echo "[model] model:  ${TARGET_DIR}/model.int8.onnx"
echo "[model] tokens: ${TARGET_DIR}/tokens.txt"
