#!/usr/bin/env bash
# Rerun the risingwave_sqlparser parse benches and compare against perf/baseline.json.
# Verdict: regression >10% = warn, >25% = block (exit 1), otherwise pass.
# Requires: repo-pinned toolchain (rust-toolchain file: nightly-2026-06-21) via rustup.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export RUSTUP_AUTO_INSTALL=0
TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
export CARGO_TARGET_DIR="$TARGET_DIR"

cargo bench -p risingwave_sqlparser --bench parse_bench -- --save-baseline rerun

python3 - "$TARGET_DIR" <<'EOF'
import json, sys
target = sys.argv[1]
base = json.load(open("perf/baseline.json"))
worst = "pass"
for b in base["benchmarks"]:
    short = b["name"].split("/")[-1]
    est = json.load(open(f"{target}/criterion/sqlparser_parse/{short}/rerun/estimates.json"))
    new = est["mean"]["point_estimate"]
    old = b["mean_ns"]
    delta = (new - old) / old * 100.0
    if delta > 25:
        v = "block"
    elif delta > 10:
        v = "warn"
    else:
        v = "pass"
    if v == "block" or (v == "warn" and worst == "pass"):
        worst = v
    kind = "regression" if delta > 0 else "improvement"
    print(f"{b['name']}: baseline {old:,.0f} ns -> rerun {new:,.0f} ns  ({delta:+.1f}% {kind})  verdict={v}")
print(f"OVERALL VERDICT: {worst}")
sys.exit(1 if worst == "block" else 0)
EOF
