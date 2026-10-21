#!/bin/bash

set -euo pipefail

# Benchmarks the Russell sparse linear solvers on the SuiteSparse matrices
# stored in ~/Downloads/matrix-market.
#
# Each run writes one JSON file per (solver, matrix) pair under
# results/<system>/<solver>/<solver>-<matrix>.json. A failed or timed-out run
# leaves no file, and the table generators render it as "n/a".
#
# Environment (required):
#   source ~/Documents/deepseek-harness/default-workspace/cargo-env.sh
#   export RUSSELL_LOCAL_PREFIX=/usr/local
#
# Optional overrides:
#   RUSSELL_DIR          Russell checkout        (default: ../russell)
#   INPUT_DIR            .mtx directory          (default: ~/Downloads/matrix-market)
#   SYSTEM               results subdirectory    (default: arch)
#   NRUN                 UMFPACK/MUMPS runs      (default: 5)
#   NATIVE_MAX_RUNS      Native run ceiling      (default: 5)
#   NATIVE_PROBE_TIMEOUT Native probe time, s    (default: 180)
#   NATIVE_MEMORY_KB     Native memory cap, KiB  (default: 20000000, ~20 GB)
#   NATIVE_MAX_NNZ       Skip Native above this pattern size; 0 disables it (default: 0)
#   STAGES               all | external | native (default: all)

NRUN=${NRUN:-5}
NATIVE_MAX_RUNS=${NATIVE_MAX_RUNS:-5}
NATIVE_PROBE_TIMEOUT=${NATIVE_PROBE_TIMEOUT:-180}
NATIVE_MEMORY_KB=${NATIVE_MEMORY_KB:-20000000}
NATIVE_MAX_NNZ=${NATIVE_MAX_NNZ:-0}
STAGES=${STAGES:-all}

HERE=$(cd "$(dirname "$0")" && pwd)
RUSSELL_DIR=${RUSSELL_DIR:-$HERE/../russell}
INPUT_DIR=${INPUT_DIR:-$HOME/Downloads/matrix-market}
SYSTEM=${SYSTEM:-arch}

FEATURES="intel_mkl,umfpack,mumps"

# MUMPS is compiled from the local prefix only
: "${RUSSELL_LOCAL_PREFIX:?export RUSSELL_LOCAL_PREFIX=/usr/local before running this script}"

# Build the CLI from the local Russell checkout
(
    cd "$RUSSELL_DIR/russell_sparse"
    cargo build --release --features "$FEATURES"
)
TARGET_DIR=${CARGO_TARGET_DIR:-$RUSSELL_DIR/target}
COMMAND="$TARGET_DIR/release/solve_matrix_market"
if [ ! -x "$COMMAND" ]; then
    echo "error: cannot find $COMMAND"
    exit 1
fi

OUTPUT_DIR="$HERE/results/$SYSTEM"
mkdir -p "$OUTPUT_DIR"

# Real matrices
REAL_MATS="
    bwm2000
    rdb5000
    Goodwin_040
    fp
    xenon1
    twotone
    Raj1
    boyd2
    Goodwin_071
    darcy003
    rma10
    helm2d03
    stomach
    oilpan
    ASIC_680k
    tmt_unsym
    Goodwin_127
    pre2
    marine1
    torso1
    atmosmodd
    atmosmodl
    memchip
    Freescale1
    rajat31
    Transport
    inline_1
    PFlow_742
    Emilia_923
    dielFilterV2real
    Flan_1565
    pres-cylin-3d-tet10-fine
"

# Complex matrices
COMPLEX_MATS="
    mhd1280b
    mplate
    RFdevice
    vfem
    fem_filter
    Chevron4
    mono_500Hz
    kim2
    fem_hifreq_circuit
    dielFilterV3clx
"

# Verify the inputs before starting
for mat in $REAL_MATS $COMPLEX_MATS; do
    if [ ! -f "$INPUT_DIR/$mat.mtx" ]; then
        echo "error: missing $INPUT_DIR/$mat.mtx"
        exit 1
    fi
done

# Extra arguments for a given matrix
extra_args() {
    case "$1" in
        oilpan | inline_1 | PFlow_742 | Emilia_923 | Flan_1565) echo "-p" ;;
        *) echo "" ;;
    esac
}

# Number of pattern entries of a matrix, read from data/matrices.json
get_pattern_entries() {
    python3 -c 'import json,sys
data = json.load(open(sys.argv[2]))
print(next((m["Pattern Entries"] or 0 for m in data if m["Name"] == sys.argv[1]), 0))' "$1" "$HERE/data/matrices.json"
}

# Runs UMFPACK or MUMPS on a list of matrices
run_external() {
    local genie=$1
    local list=$2
    local outdir="$OUTPUT_DIR/$genie"
    mkdir -p "$outdir"
    for mat in $list; do
        local extra
        extra=$(extra_args "$mat")
        echo "... $genie / $mat ..."
        if ! timeout 3600 "$COMMAND" -g "$genie" -r "$NRUN" $extra "$INPUT_DIR/$mat.mtx" > "$outdir/$genie-$mat.json"; then
            echo "    failed or timed out; removing the partial result"
            rm -f "$outdir/$genie-$mat.json"
        fi
    done
}

# Runs Native on one matrix, with a probe pass to decide how many runs are worth doing
run_native_matrix() {
    local mat=$1
    local extra
    extra=$(extra_args "$mat")
    local outdir="$OUTPUT_DIR/native"
    local probe="/tmp/benchmark-russell/native-probe-$mat.json"
    mkdir -p "$outdir" /tmp/benchmark-russell

    local nnz
    nnz=$(get_pattern_entries "$mat")
    if [ "$NATIVE_MAX_NNZ" -gt 0 ] && [ "$nnz" -gt "$NATIVE_MAX_NNZ" ]; then
        echo "... native / $mat ... skipping (pattern entries $nnz > $NATIVE_MAX_NNZ)"
        rm -f "$outdir/native-$mat.json"
        return
    fi

    echo "... native / $mat (probe) ..."
    local t0 t1 elapsed
    t0=$(date +%s%N)
    if (
        ulimit -v "$NATIVE_MEMORY_KB"
        exec timeout "$NATIVE_PROBE_TIMEOUT" "$COMMAND" -g native -r 1 $extra "$INPUT_DIR/$mat.mtx"
    ) > "$probe" 2> /dev/null; then
        t1=$(date +%s%N)
        elapsed=$(((t1 - t0) / 1000000000))

        local nrun
        if [ "$elapsed" -lt 2 ]; then
            nrun=$NATIVE_MAX_RUNS
        elif [ "$elapsed" -lt 30 ]; then
            nrun=$((NATIVE_MAX_RUNS > 3 ? 3 : NATIVE_MAX_RUNS))
        else
            nrun=1
        fi

        if [ "$nrun" -eq 1 ]; then
            mv "$probe" "$outdir/native-$mat.json"
            echo "    probe took ${elapsed}s; keeping the single run"
        else
            echo "    probe took ${elapsed}s; running $nrun times"
            if ! (
                ulimit -v "$NATIVE_MEMORY_KB"
                exec timeout $((elapsed * nrun * 3 + 600)) "$COMMAND" -g native -r "$nrun" $extra "$INPUT_DIR/$mat.mtx"
            ) > "$outdir/native-$mat.json" 2> /dev/null; then
                echo "    full run failed or timed out; keeping the probe result"
                mv "$probe" "$outdir/native-$mat.json"
            else
                rm -f "$probe"
            fi
        fi
    else
        echo "    probe failed or timed out; marking n/a"
        rm -f "$probe" "$outdir/native-$mat.json"
    fi
}

run_native() {
    local list=$1
    for mat in $list; do
        run_native_matrix "$mat"
    done
}

# External solvers first so the long Native pass can be interrupted without
# losing their results
if [ "$STAGES" = "all" ] || [ "$STAGES" = "external" ]; then
    run_external umfpack "$REAL_MATS"
    run_external umfpack "$COMPLEX_MATS"
    run_external mumps "$REAL_MATS"
    run_external mumps "$COMPLEX_MATS"
fi

if [ "$STAGES" = "all" ] || [ "$STAGES" = "native" ]; then
    run_native "$REAL_MATS"
    run_native "$COMPLEX_MATS"
fi

echo "done: results in $OUTPUT_DIR"
