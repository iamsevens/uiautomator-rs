#!/usr/bin/env bash
set -euo pipefail

allow_dirty=""
if [[ "${1:-}" == "--allow-dirty" ]]; then
  allow_dirty="--allow-dirty"
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

forbidden_regex='(^|/)(TASK_.*\.md|.*_REPORT\.md|.*_SUMMARY\.md|MANUAL_TEST_.*\.md)$'

check_crate() {
  local crate_name="$1"
  local crate_path="$2"
  shift 2
  local required_files=("$@")

  echo ""
  echo "==> Checking package list for ${crate_name}"

  pushd "$crate_path" >/dev/null
  local package_list
  package_list="$(cargo package ${allow_dirty} --list | sed 's#\\#/#g')"
  popd >/dev/null

  if echo "$package_list" | grep -Eiq "$forbidden_regex"; then
    echo "Forbidden files found in ${crate_name} package:"
    echo "$package_list" | grep -Ei "$forbidden_regex"
    exit 1
  fi

  if echo "$package_list" | grep -Eiq '^(tests|examples)/'; then
    echo "Non-release content found in ${crate_name} package:"
    echo "$package_list" | grep -Ei '^(tests|examples)/'
    exit 1
  fi

  local required
  for required in "${required_files[@]}"; do
    if ! echo "$package_list" | grep -Fxq "$required"; then
      echo "Required file missing from ${crate_name} package: ${required}"
      exit 1
    fi
  done

  echo "OK: ${crate_name} package list passed."
}

check_crate "uiautomator" "uiautomator" \
  "Cargo.toml" \
  "README.md" \
  "THIRD_PARTY_NOTICES.md" \
  "build.rs" \
  "src/lib.rs" \
  "assets/u2.jar"

check_crate "uiautomator-cli" "uiautomator-cli" \
  "Cargo.toml" \
  "README.md" \
  "CHANGELOG.md" \
  "THIRD_PARTY_NOTICES.md" \
  "build.rs" \
  "src/main.rs" \
  "assets/atx-agent" \
  "assets/app-uiautomator.apk" \
  "assets/app-uiautomator-test.apk"

echo ""
echo "All package-list checks passed."
