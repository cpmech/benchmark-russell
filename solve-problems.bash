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
#   DIRECT_MAX_RUNS      Native run ceiling      (default: 5)
#   DIRECT_PROBE_TIMEOUT Native probe time, s    (default: 180)
#   DIRECT_MEMORY_KB     Native memory cap, KiB  (default: 20000000, ~20 GB)
#   DIRECT_MAX_NNZ       Skip Native above this pattern size; 0 disables it (default: 0)
#   DIRECT_SUPERNODAL    1 selects the experimental supernodal LU (default: 0)
#   STAGES               all | external | direct | iterative (default: all)

NRUN=${NRUN:-5}
DIRECT_MAX_RUNS=${DIRECT_MAX_RUNS:-5}
DIRECT_PROBE_TIMEOUT=${DIRECT_PROBE_TIMEOUT:-180}
DIRECT_MEMORY_KB=${DIRECT_MEMORY_KB:-20000000}
DIRECT_MAX_NNZ=${DIRECT_MAX_NNZ:-0}
DIRECT_SUPERNODAL=${DIRECT_SUPERNODAL:-0}
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
    pmsim-hex8-27
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
        oilpan | inline_1 | PFlow_742 | Emilia_923 | Flan_1565 | pres-cylin-3d-tet10-fine) echo "-p" ;;
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

# Runs the Direct solver on one matrix, with a probe pass to decide how many runs are worth doing
run_direct_matrix() {
    local mat=$1
    local extra
    extra=$(extra_args "$mat")
    local outdir="$OUTPUT_DIR/direct"
    local direct_flag=""
    if [ "$DIRECT_SUPERNODAL" = "1" ]; then
        outdir="$OUTPUT_DIR/direct-supernodal"
        direct_flag="--direct-supernodal"
    fi
    local probe="/tmp/benchmark-russell/direct-probe-$mat.json"
    mkdir -p "$outdir" /tmp/benchmark-russell

    local nnz
    nnz=$(get_pattern_entries "$mat")
    if [ "$DIRECT_MAX_NNZ" -gt 0 ] && [ "$nnz" -gt "$DIRECT_MAX_NNZ" ]; then
        echo "... direct / $mat ... skipping (pattern entries $nnz > $DIRECT_MAX_NNZ)"
        rm -f "$outdir/direct-$mat.json"
        return
    fi

    echo "... direct / $mat (probe) ..."
    local t0 t1 elapsed
    t0=$(date +%s%N)
    if (
        ulimit -v "$DIRECT_MEMORY_KB"
        exec timeout "$DIRECT_PROBE_TIMEOUT" "$COMMAND" -g direct -r 1 $direct_flag $extra "$INPUT_DIR/$mat.mtx"
    ) > "$probe" 2> /dev/null; then
        t1=$(date +%s%N)
        elapsed=$(((t1 - t0) / 1000000000))

        local nrun
        if [ "$elapsed" -lt 2 ]; then
            nrun=$DIRECT_MAX_RUNS
        elif [ "$elapsed" -lt 30 ]; then
            nrun=$((DIRECT_MAX_RUNS > 3 ? 3 : DIRECT_MAX_RUNS))
        else
            nrun=1
        fi

        if [ "$nrun" -eq 1 ]; then
            mv "$probe" "$outdir/direct-$mat.json"
            echo "    probe took ${elapsed}s; keeping the single run"
        else
            echo "    probe took ${elapsed}s; running $nrun times"
            if ! (
                ulimit -v "$DIRECT_MEMORY_KB"
                exec timeout $((elapsed * nrun * 3 + 600)) "$COMMAND" -g direct -r "$nrun" $direct_flag $extra "$INPUT_DIR/$mat.mtx"
            ) > "$outdir/direct-$mat.json" 2> /dev/null; then
                echo "    full run failed or timed out; keeping the probe result"
                mv "$probe" "$outdir/direct-$mat.json"
            else
                rm -f "$probe"
            fi
        fi
    else
        echo "    probe failed or timed out; marking n/a"
        rm -f "$probe" "$outdir/direct-$mat.json"
    fi
}

run_direct() {
    local list=$1
    for mat in $list; do
        run_direct_matrix "$mat"
    done
}

# Runs one Krylov method through russell_sparse's iterative solver
run_iterative() {
    local genie=$1
    local list=$2
    local outdir="$OUTPUT_DIR/$genie"
    mkdir -p "$outdir"
    for mat in $list; do
        echo "... $genie / $mat ..."
        if ! timeout 3600 "$COMMAND" -g "$genie" -r "$NRUN" "$INPUT_DIR/$mat.mtx" > "$outdir/$genie-$mat.json"; then
            echo "    failed or timed out; removing the partial result"
            rm -f "$outdir/$genie-$mat.json"
        fi
    done
}

# External solvers first so the long Direct pass can be interrupted without
# losing their results
if [ "$STAGES" = "all" ] || [ "$STAGES" = "external" ]; then
    run_external umfpack "$REAL_MATS"
    run_external umfpack "$COMPLEX_MATS"
    run_external mumps "$REAL_MATS"
    run_external mumps "$COMPLEX_MATS"
fi

if [ "$STAGES" = "all" ] || [ "$STAGES" = "direct" ]; then
    run_direct "$REAL_MATS"
    run_direct "$COMPLEX_MATS"
fi

# The iterative solvers are real-valued only: CG needs a symmetric matrix, and the
# preconditioner is built from the same real pattern
if [ "$STAGES" = "all" ] || [ "$STAGES" = "iterative" ]; then
    run_iterative cg "$REAL_MATS"
    run_iterative bicgstab "$REAL_MATS"
    run_iterative gmres "$REAL_MATS"
fi

echo "done: results in $OUTPUT_DIR"
