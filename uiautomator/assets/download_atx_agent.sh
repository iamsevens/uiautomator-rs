#!/usr/bin/env bash
# Download ATX-Agent binaries (multi-arch) and UiAutomator APK assets.

set -euo pipefail

ASSETS_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ASSETS_DIR"

ATX_AGENT_VERSION="${ATX_AGENT_VERSION:-0.10.0}"
UIAUTOMATOR_VERSION="${UIAUTOMATOR_VERSION:-2.3.6}"

sha256_file() {
  local file_path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file_path" | awk '{print $1}'
    return
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file_path" | awk '{print $1}'
    return
  fi
  echo "No SHA256 tool found (need sha256sum or shasum)" >&2
  exit 1
}

expected_atx_sha256() {
  local arch="$1"
  case "${ATX_AGENT_VERSION}:${arch}" in
    "0.10.0:armv7") echo "4157ec30b7125266370782e03eba53edfee1e719dc8572c3e9565c212668b0f8" ;;
    "0.10.0:arm64") echo "458bc5bacaae32abbe658262257b1a42345a566c684f93babd2dc0778ca6d78f" ;;
    "0.10.0:amd64") echo "e338480e34fdaa9f0bedbf8d9c7e6c15e1335805c0e1c6d1d209f528590be3c9" ;;
    "0.10.0:386") echo "bfde550ff7fdfe4926d96f6d23d15ace099cec1be9e2c52455efc8119a97f8a7" ;;
    *) echo "" ;;
  esac
}

expected_apk_sha256() {
  local file_name="$1"
  case "${UIAUTOMATOR_VERSION}:${file_name}" in
    "2.3.6:app-uiautomator.apk") echo "6f85594700ad96de89d012b3767049c2c6988510b68b31b439dd2a6dd93a30c9" ;;
    "2.3.6:app-uiautomator-test.apk") echo "b768dfa7085389234feffc9246275ad5c3301db98424634bd9e06d916df0e3e4" ;;
    *) echo "" ;;
  esac
}

verify_checksum() {
  local file_name="$1"
  local expected="$2"
  if [[ -z "$expected" ]]; then
    echo "Skip checksum for ${file_name} (no expected hash for selected version)"
    return
  fi

  local actual
  actual="$(sha256_file "$file_name" | tr '[:upper:]' '[:lower:]')"
  if [[ "$actual" != "$expected" ]]; then
    echo "Checksum mismatch: ${file_name}" >&2
    echo "  expected: ${expected}" >&2
    echo "  actual  : ${actual}" >&2
    exit 1
  fi
}

download_atx_agent() {
  local arch="$1"
  local output_name="$2"
  local archive_name="atx-agent_${ATX_AGENT_VERSION}_linux_${arch}.tar.gz"
  local url="https://github.com/openatx/atx-agent/releases/download/${ATX_AGENT_VERSION}/${archive_name}"

  echo "Downloading ${archive_name} ..."
  curl -fL "$url" -o "$archive_name"
  tar -xzf "$archive_name" atx-agent
  mv -f atx-agent "$output_name"
  chmod +x "$output_name"
  rm -f "$archive_name"
  verify_checksum "$output_name" "$(expected_atx_sha256 "$arch")"
}

download_file_if_missing() {
  local file_name="$1"
  local url="$2"

  if [[ -f "$file_name" ]]; then
    echo "Skip ${file_name} (already exists)"
    verify_checksum "$file_name" "$(expected_apk_sha256 "$file_name")"
    return
  fi

  echo "Downloading ${file_name} ..."
  curl -fL "$url" -o "$file_name"

  verify_checksum "$file_name" "$(expected_apk_sha256 "$file_name")"
}

echo "Downloading ATX-Agent assets (version ${ATX_AGENT_VERSION}) ..."
download_atx_agent "armv7" "atx-agent-armv7"
download_atx_agent "arm64" "atx-agent-arm64"
download_atx_agent "amd64" "atx-agent-amd64"
download_atx_agent "386" "atx-agent-386"

# Keep the legacy filename for backward compatibility with older tooling.
cp -f atx-agent-armv7 atx-agent
verify_checksum "atx-agent" "$(expected_atx_sha256 "armv7")"

echo "Downloading UiAutomator APK assets (version ${UIAUTOMATOR_VERSION}) ..."
download_file_if_missing \
  "app-uiautomator.apk" \
  "https://github.com/openatx/android-uiautomator-server/releases/download/${UIAUTOMATOR_VERSION}/app-uiautomator.apk"
download_file_if_missing \
  "app-uiautomator-test.apk" \
  "https://github.com/openatx/android-uiautomator-server/releases/download/${UIAUTOMATOR_VERSION}/app-uiautomator-test.apk"

echo
echo "Done."
ls -lh atx-agent atx-agent-armv7 atx-agent-arm64 atx-agent-amd64 atx-agent-386 app-uiautomator.apk app-uiautomator-test.apk
