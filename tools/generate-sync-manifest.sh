#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-spec-cli}"
TARGET_REPO="${2:-}"
OUT_PATH="${3:-$ROOT_DIR/sync-manifest.${MODE}.json}"

die() {
  echo "error: $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

jesc() {
  printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'
}

dir_file_count() {
  find "$1" -type f | wc -l | tr -d ' '
}

file_sha256() {
  shasum -a 256 "$1" | awk '{print $1}'
}

dir_manifest_sha256() {
  local dir="$1"
  local tmp
  tmp="$(mktemp)"
  (
    cd "$dir"
    find . -type f -print | LC_ALL=C sort
  ) | while IFS= read -r rel; do
    # strip leading ./ for stable readability
    rel="${rel#./}"
    hash="$(shasum -a 256 "$dir/$rel" | awk '{print $1}')"
    printf '%s  %s\n' "$hash" "$rel"
  done >"$tmp"
  shasum -a 256 "$tmp" | awk '{print $1}'
  rm -f "$tmp"
}

require_cmd git
require_cmd find
require_cmd shasum
require_cmd sed
require_cmd awk

[[ -n "$TARGET_REPO" ]] || die "usage: $0 <spec-cli|openapi> <target-repo-path> [output-json]"
[[ -d "$TARGET_REPO/.git" ]] || die "target repo path must be a git checkout: $TARGET_REPO"

source_repo_name="provenact-spec"
source_commit="$(git -C "$ROOT_DIR" rev-parse HEAD)"
target_repo_name="$(basename "$TARGET_REPO")"
target_commit="$(git -C "$TARGET_REPO" rev-parse HEAD)"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

case "$MODE" in
  spec-cli)
    src_spec="$ROOT_DIR/spec"
    src_vectors="$ROOT_DIR/test-vectors"
    dst_spec="$TARGET_REPO/spec"
    dst_vectors="$TARGET_REPO/test-vectors"
    [[ -d "$dst_spec" ]] || die "missing mirror path: $dst_spec"
    [[ -d "$dst_vectors" ]] || die "missing mirror path: $dst_vectors"

    src_spec_digest="$(dir_manifest_sha256 "$src_spec")"
    dst_spec_digest="$(dir_manifest_sha256 "$dst_spec")"
    src_vectors_digest="$(dir_manifest_sha256 "$src_vectors")"
    dst_vectors_digest="$(dir_manifest_sha256 "$dst_vectors")"
    src_spec_count="$(dir_file_count "$src_spec")"
    dst_spec_count="$(dir_file_count "$dst_spec")"
    src_vectors_count="$(dir_file_count "$src_vectors")"
    dst_vectors_count="$(dir_file_count "$dst_vectors")"

    cat >"$OUT_PATH" <<EOF
{
  "schema_version": "1.0",
  "mode": "spec-cli",
  "generated_at_utc": "$(jesc "$generated_at")",
  "source_repo": {
    "name": "$(jesc "$source_repo_name")",
    "path": "$(jesc "$ROOT_DIR")",
    "commit": "$(jesc "$source_commit")"
  },
  "target_repo": {
    "name": "$(jesc "$target_repo_name")",
    "path": "$(jesc "$TARGET_REPO")",
    "commit": "$(jesc "$target_commit")"
  },
  "artifacts": [
    {
      "name": "spec",
      "source_path": "spec",
      "target_path": "spec",
      "source_file_count": $src_spec_count,
      "target_file_count": $dst_spec_count,
      "source_manifest_sha256": "$(jesc "$src_spec_digest")",
      "target_manifest_sha256": "$(jesc "$dst_spec_digest")",
      "in_sync": $( [[ "$src_spec_digest" == "$dst_spec_digest" ]] && echo true || echo false )
    },
    {
      "name": "test-vectors",
      "source_path": "test-vectors",
      "target_path": "test-vectors",
      "source_file_count": $src_vectors_count,
      "target_file_count": $dst_vectors_count,
      "source_manifest_sha256": "$(jesc "$src_vectors_digest")",
      "target_manifest_sha256": "$(jesc "$dst_vectors_digest")",
      "in_sync": $( [[ "$src_vectors_digest" == "$dst_vectors_digest" ]] && echo true || echo false )
    }
  ]
}
EOF
    ;;

  openapi)
    source_repo_name="provenact-control"
    src_openapi="${OPENAPI_SOURCE_FILE:-$ROOT_DIR/../provenact-control/openapi.yaml}"
    dst_openapi="$TARGET_REPO/public/openapi.yaml"
    [[ -f "$src_openapi" ]] || die "missing source openapi file: $src_openapi"
    [[ -f "$dst_openapi" ]] || die "missing target openapi file: $dst_openapi"
    source_commit="$(git -C "$(dirname "$src_openapi")" rev-parse HEAD 2>/dev/null || echo unknown)"
    src_openapi_sha="$(file_sha256 "$src_openapi")"
    dst_openapi_sha="$(file_sha256 "$dst_openapi")"

    cat >"$OUT_PATH" <<EOF
{
  "schema_version": "1.0",
  "mode": "openapi",
  "generated_at_utc": "$(jesc "$generated_at")",
  "source_repo": {
    "name": "$(jesc "$source_repo_name")",
    "path": "$(jesc "$(dirname "$src_openapi")")",
    "commit": "$(jesc "$source_commit")"
  },
  "target_repo": {
    "name": "$(jesc "$target_repo_name")",
    "path": "$(jesc "$TARGET_REPO")",
    "commit": "$(jesc "$target_commit")"
  },
  "artifacts": [
    {
      "name": "openapi.yaml",
      "source_path": "$(jesc "$src_openapi")",
      "target_path": "public/openapi.yaml",
      "source_sha256": "$(jesc "$src_openapi_sha")",
      "target_sha256": "$(jesc "$dst_openapi_sha")",
      "in_sync": $( [[ "$src_openapi_sha" == "$dst_openapi_sha" ]] && echo true || echo false )
    }
  ]
}
EOF
    ;;

  *)
    die "unknown mode: $MODE (expected spec-cli or openapi)"
    ;;
esac

echo "ok: wrote sync manifest to $OUT_PATH"
