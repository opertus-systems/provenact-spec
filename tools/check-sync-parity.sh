#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-spec-cli}"
TARGET_REPO="${2:-}"

die() {
  echo "error: $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

require_cmd diff
require_cmd find

if [[ -z "$TARGET_REPO" ]]; then
  die "usage: $0 <spec-cli|openapi> <target-repo-path>"
fi

if [[ ! -d "$TARGET_REPO/.git" ]]; then
  die "target repo path must be a git checkout: $TARGET_REPO"
fi

case "$MODE" in
  spec-cli)
    SRC_SPEC="$ROOT_DIR/spec"
    SRC_VECTORS="$ROOT_DIR/test-vectors"
    DST_SPEC="$TARGET_REPO/spec"
    DST_VECTORS="$TARGET_REPO/test-vectors"

    [[ -d "$DST_SPEC" ]] || die "missing mirror path: $DST_SPEC"
    [[ -d "$DST_VECTORS" ]] || die "missing mirror path: $DST_VECTORS"

    echo "checking spec parity..."
    diff -qr "$SRC_SPEC" "$DST_SPEC"

    echo "checking test-vectors parity..."
    diff -qr "$SRC_VECTORS" "$DST_VECTORS"

    src_spec_files="$(find "$SRC_SPEC" -type f | wc -l | tr -d ' ')"
    dst_spec_files="$(find "$DST_SPEC" -type f | wc -l | tr -d ' ')"
    src_vector_files="$(find "$SRC_VECTORS" -type f | wc -l | tr -d ' ')"
    dst_vector_files="$(find "$DST_VECTORS" -type f | wc -l | tr -d ' ')"

    echo "ok: spec parity"
    echo "spec files: source=$src_spec_files mirror=$dst_spec_files"
    echo "test-vectors files: source=$src_vector_files mirror=$dst_vector_files"
    ;;

  openapi)
    SRC_OPENAPI="$ROOT_DIR/../provenact-control/openapi.yaml"
    DST_OPENAPI="$TARGET_REPO/public/openapi.yaml"

    [[ -f "$SRC_OPENAPI" ]] || die "missing source openapi: $SRC_OPENAPI"
    [[ -f "$DST_OPENAPI" ]] || die "missing mirror openapi: $DST_OPENAPI"

    diff -q "$SRC_OPENAPI" "$DST_OPENAPI"
    echo "ok: openapi parity"
    ;;

  *)
    die "unknown mode: $MODE (expected spec-cli or openapi)"
    ;;
esac
