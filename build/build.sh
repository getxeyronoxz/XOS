#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${REPO_ROOT}/output"
IMAGE_NAME="${XOS_BUILDER_IMAGE:-xos-builder}"

echo "==> Building Docker image: ${IMAGE_NAME}"
docker build -t "${IMAGE_NAME}" -f "${REPO_ROOT}/build/Dockerfile" "${REPO_ROOT}"

mkdir -p "${OUTPUT_DIR}"

echo "==> Running ISO build container"
docker run --rm \
    --privileged \
    -v "${OUTPUT_DIR}:/xos/output" \
    "${IMAGE_NAME}"

echo "==> Build complete. ISO in ${OUTPUT_DIR}/"
