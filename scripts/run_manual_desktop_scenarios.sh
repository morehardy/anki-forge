#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MANIFEST_PATH="${REPO_ROOT}/contracts/manifest.yaml"
SCENARIO_ROOT="${REPO_ROOT}/contracts/fixtures/phase3/manual-desktop-v1"
OUTPUT_ROOT="${REPO_ROOT}/tmp/manual-desktop-v1"

SCENARIOS=(
  "S01_basic_text_minimal"
  "S02_cloze_minimal"
  "S03_io_minimal"
  "S04_basic_image"
  "S05_basic_audio"
  "S06_basic_video"
  "S07_cloze_mixed_media"
  "S08_io_plus_audio"
  "S09_io_rect"
)

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run_manual_desktop_scenarios.sh
  ./scripts/run_manual_desktop_scenarios.sh <scenario> [<scenario> ...]

Examples:
  ./scripts/run_manual_desktop_scenarios.sh
  ./scripts/run_manual_desktop_scenarios.sh S05_basic_audio
EOF
}

is_known_scenario() {
  local target="$1"
  local item
  for item in "${SCENARIOS[@]}"; do
    if [[ "${item}" == "${target}" ]]; then
      return 0
    fi
  done
  return 1
}

run_one() {
  local scene="$1"
  local input_path="${SCENARIO_ROOT}/${scene}/input/authoring-ir.json"
  local out_dir="${OUTPUT_ROOT}/${scene}"
  local normalize_result_path="${out_dir}/normalize.result.json"
  local normalized_ir_path="${out_dir}/normalized-ir.json"

  if [[ ! -f "${input_path}" ]]; then
    echo "missing scenario input: ${input_path}" >&2
    return 1
  fi

  mkdir -p "${out_dir}"

  echo "==> ${scene}: normalize"
  cargo run -q -p contract_tools -- normalize \
    --manifest "${MANIFEST_PATH}" \
    --input "${input_path}" \
    --output contract-json > "${normalize_result_path}"

  jq -e '.normalized_ir' "${normalize_result_path}" > "${normalized_ir_path}"

  echo "==> ${scene}: build"
  cargo run -q -p contract_tools -- build \
    --manifest "${MANIFEST_PATH}" \
    --input "${normalized_ir_path}" \
    --writer-policy default \
    --build-context default \
    --artifacts-dir "${out_dir}" \
    --output contract-json > "${out_dir}/build.json"

  echo "==> ${scene}: inspect apkg"
  cargo run -q -p contract_tools -- inspect \
    --apkg "${out_dir}/package.apkg" \
    --output contract-json > "${out_dir}/apkg.inspect.json"

  echo "done: ${out_dir}/package.apkg"
}

main() {
  if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
  fi

  local targets=()
  if [[ "$#" -eq 0 ]]; then
    targets=("${SCENARIOS[@]}")
  else
    local arg
    for arg in "$@"; do
      if ! is_known_scenario "${arg}"; then
        echo "unknown scenario: ${arg}" >&2
        echo "known scenarios: ${SCENARIOS[*]}" >&2
        exit 1
      fi
      targets+=("${arg}")
    done
  fi

  mkdir -p "${OUTPUT_ROOT}"
  local t
  for t in "${targets[@]}"; do
    run_one "${t}"
  done
}

main "$@"
