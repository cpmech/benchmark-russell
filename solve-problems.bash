#!/bin/bash

set -euo pipefail

# Benchmarks the Russell sparse linear solvers on the SuiteSparse matrices in
# ~/Downloads/matrix-market, writing one JSON file per (solver, matrix) under
# results/<system>/<solver>/<solver>-<matrix>.json. A failed, timed-out or
# contaminated run leaves no file, and the table generators render it "n/a".
#
# Environment (required):
#   export RUSSELL_LOCAL_PREFIX=/usr/local
#
# Optional overrides:
#   RUSSELL_DIR          Russell checkout        (default: ../russell)
#   INPUT_DIR            .mtx directory          (default: ~/Downloads/matrix-market)
#   SYSTEM               results subdirectory    (default: arch)
#   NRUN                 runs per solver         (default: 3)
#   EXTERNAL_TIMEOUT     UMFPACK/MUMPS per-run time cap, s (default: 720)
#   DIRECT_MAX_RUNS      Native run ceiling      (default: 3)
#   DIRECT_PROBE_TIMEOUT Native probe time, s    (default: 180)
#   DIRECT_MEMORY_KB     Native memory cap, KiB  (default: 20000000, ~20 GB)
#   DIRECT_MAX_NNZ       Skip Native above this pattern size; 0 disables it (default: 1200000)
#   DIRECT_SUPERNODAL    1 selects the experimental supernodal LU (default: 0)
#   STAGES               all | external | direct | iterative (default: all)
#   ITERATIVE_MEMORY_KB  per-run memory cap, KiB (default: 8000000, ~8 GB)
#   ITERATIVE_TIMEOUT    per-run time cap, s     (default: 240)
#   TIMEOUT_OVERHEAD     batch read/verify allowance, s (default: 60)

NRUN=${NRUN:-3}
EXTERNAL_TIMEOUT=${EXTERNAL_TIMEOUT:-720}
DIRECT_MAX_RUNS=${DIRECT_MAX_RUNS:-3}
DIRECT_PROBE_TIMEOUT=${DIRECT_PROBE_TIMEOUT:-180}
DIRECT_MEMORY_KB=${DIRECT_MEMORY_KB:-20000000}
DIRECT_MAX_NNZ=${DIRECT_MAX_NNZ:-1200000}
DIRECT_SUPERNODAL=${DIRECT_SUPERNODAL:-0}
STAGES=${STAGES:-all}
ITERATIVE_MEMORY_KB=${ITERATIVE_MEMORY_KB:-8000000}
ITERATIVE_TIMEOUT=${ITERATIVE_TIMEOUT:-240}
TIMEOUT_OVERHEAD=${TIMEOUT_OVERHEAD:-60}

HERE=$(cd "$(dirname "$0")" && pwd)
RUSSELL_DIR=${RUSSELL_DIR:-$HERE/../russell}
INPUT_DIR=${INPUT_DIR:-$HOME/Downloads/matrix-market}
SYSTEM=${SYSTEM:-arch}
OUTPUT_DIR=$HERE/results/$SYSTEM
FEATURES="intel_mkl,umfpack,mumps"

# MUMPS is compiled from the local prefix only
: "${RUSSELL_LOCAL_PREFIX:?export RUSSELL_LOCAL_PREFIX=/usr/local before running this script}"

# Matrices in ascending size, so an interrupted suite still has the small rows
REAL_MATS=(
    bwm2000 rdb5000 Goodwin_040 fp
    xenon1 twotone Raj1 boyd2
    Goodwin_071 darcy003 rma10 helm2d03
    stomach oilpan ASIC_680k tmt_unsym
    Goodwin_127 pre2 marine1 torso1
    atmosmodd atmosmodl memchip Freescale1
    rajat31 Transport inline_1 PFlow_742
    Emilia_923 dielFilterV2real Flan_1565
    pres-cylin-3d-tet10-fine pmsim-hex8-27
)
COMPLEX_MATS=(
    mhd1280b mplate RFdevice vfem
    fem_filter Chevron4 mono_500Hz kim2
    fem_hifreq_circuit dielFilterV3clx
)

# Build the CLI from the local Russell checkout
(cd "$RUSSELL_DIR/russell_sparse" && cargo build --release --features "$FEATURES")
# cargo metadata reports the effective target directory, CARGO_TARGET_DIR included
TARGET_DIR=$(
    cd "$RUSSELL_DIR/russell_sparse"
    cargo metadata --format-version 1 --no-deps |
        python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'
)
COMMAND=$TARGET_DIR/release/solve_matrix_market
if [ ! -x "$COMMAND" ]; then
    echo "error: cannot find $COMMAND"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

# --------------------------------------------------------------------------
# Memory guard
# --------------------------------------------------------------------------
# A MUMPS solve reserves a workspace several times the factor size: on the
# Transport matrix INFOG(17) estimates 39 GB for the in-core factorization and
# the solve reports 33.7 GB used, with ICNTL(14) = 100 % memory relaxation and
# out-of-core off. A 32 GB workstation serves that reservation lazily, so an
# idle run touches only a few GB and is fast; once free memory falls behind the
# reservation the solve pages and its timing grows by one to two orders of
# magnitude (0.33 s measured idle, 35 s with 20 GB held by another process),
# while the compute-bound factorization grows only two to three times. A result
# is therefore only meaningful on a machine that is not swapping, and the guard
# below enforces that instead of trusting the operator: it refuses to start a
# matrix when free memory is short, and it discards a result whose own run
# pushed pages into swap.
MEMORY_LOG="$OUTPUT_DIR/memory.log"
MIN_AVAILABLE_GB=${MIN_AVAILABLE_GB:-20}        # refuse to start below this much free memory
MAX_SWAP_GROWTH_MB=${MAX_SWAP_GROWTH_MB:-256}   # a run that grows swap by more is contaminated
MAX_SWAP_PAGES=${MAX_SWAP_PAGES:-4096}          # a run that pages more than this (16 MB) is contaminated
MIN_CGROUP_MB=${MIN_CGROUP_MB:-24000}           # refuse to time inside a memory-capped job scope
MAX_MAJFLT=${MAX_MAJFLT:-1024}                  # a run that takes more major faults than this is contaminated

mem_available_mb() { awk '/^MemAvailable/{print int($2 / 1024)}' /proc/meminfo; }
swap_used_mb()     { awk '/^SwapTotal/{t=$2} /^SwapFree/{f=$2} END{print int((t - f) / 1024)}' /proc/meminfo; }
# Cumulative page counters: net swap usage misses a run whose pages are swapped out and read back.
swap_pages()       { awk '/^pswpin/{i=$2} /^pswpout/{o=$2} END{print i + o}' /proc/vmstat; }
# The cgroup ceiling, not only the host's free memory: a job scope of a few GiB fakes a 25x solve.
cgroup_memory_max_mb() {
    local rel f v
    rel=$(awk -F: '/^0::/{print $3}' /proc/self/cgroup)
    f="/sys/fs/cgroup${rel}/memory.max"
    if [ -r "$f" ] && v=$(cat "$f") && [ "$v" != max ]; then
        echo $((v / 1024 / 1024))
    else
        echo 0
    fi
}

memory_log() {
    echo "$(date -Is) $* avail_mb=$(mem_available_mb) swap_used_mb=$(swap_used_mb)" >> "$MEMORY_LOG"
}

# require_memory LABEL: abort the stage when the machine cannot host a solve
require_memory() {
    local avail cg
    avail=$(mem_available_mb)
    memory_log "check $1"
    if [ "$avail" -lt $((MIN_AVAILABLE_GB * 1024)) ]; then
        {
            echo "error: only ${avail} MB available, below MIN_AVAILABLE_GB=${MIN_AVAILABLE_GB} GB"
            echo "       MUMPS reserves up to 39 GB here, and a solve that starts swapping is 50-100x"
            echo "       slower, so its timing is worthless. Reclaim memory (reset the swap, stop other"
            echo "       jobs) or lower MIN_AVAILABLE_GB deliberately; see results/*/memory.log."
        } >&2
        exit 1
    fi
    cg=$(cgroup_memory_max_mb)
    memory_log "cgroup_max_mb=$cg"
    if [ "$cg" -gt 0 ] && [ "$cg" -lt "$MIN_CGROUP_MB" ]; then
        {
            echo "error: this process is capped by its cgroup to ${cg} MB (memory.max), below"
            echo "       MIN_CGROUP_MB=${MIN_CGROUP_MB}. A job scope of a few GiB turns a 25 GiB"
            echo "       workspace into permanent paging: measured 6.19 M major faults and 8.9 s where"
            echo "       the same binary takes 0.37 s uncapped. Run the suite outside the capped scope."
        } >&2
        exit 1
    fi
}

# run_checked OUT LABEL MEM_KB TMO_S CMD...: run CMD with its stdout in OUT,
# under timeout TMO_S, an optional address-space cap (MEM_KB, 0 = none), and a
# major-fault bound. The fault count is the child's own, from wait4, not the
# system-wide swap figures. Populates RUN_EXIT, RUN_ELAPSED_S, RUN_MAJFLT, and
# RUN_MAXRSS_MB for memory_end logging.
run_checked() {
    local out=$1 label=$2 memkb=$3 tmo=$4 stats
    shift 4
    RUN_EXIT=1
    RUN_ELAPSED_S=0
    RUN_MAJFLT=0
    RUN_MAXRSS_MB=0
    set -- timeout "$tmo" "$@"
    if [ "$memkb" -gt 0 ]; then
        set -- bash -c 'ulimit -v "$0" || exit 9; exec "$@"' "$memkb" "$@"
    fi
    stats=$(python3 - "$out" "$label" "$tmo" "$@" <<'PY'
import os, subprocess, sys, time
out, label, tmo, cmd = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
t0 = time.monotonic()
with open(out, "w") as f:
    p = subprocess.Popen(cmd, stdout=f, stderr=subprocess.DEVNULL)
    _, status, ru = os.wait4(p.pid, 0)
elapsed = time.monotonic() - t0
rc = os.waitstatus_to_exitcode(status)
maxrss_mb = ru.ru_maxrss // 1024
print(f"    {label}: exit={rc} elapsed_s={elapsed:.1f} timeout_s={tmo} majflt={ru.ru_majflt} maxrss_mb={maxrss_mb}",
      file=sys.stderr)
print(f"{rc} {elapsed:.2f} {ru.ru_majflt} {maxrss_mb}")
PY
    )
    read -r RUN_EXIT RUN_ELAPSED_S RUN_MAJFLT RUN_MAXRSS_MB <<< "$stats"
    if [ "$RUN_MAJFLT" -gt "$MAX_MAJFLT" ]; then
        echo "    contaminated: $RUN_MAJFLT major faults during $label; discarding the result" >&2
        return 3
    fi
    [ "$RUN_EXIT" -eq 0 ]
}

# memory_begin / memory_end OUT LABEL TMO_S: log run outcome and discard a result whose run paged
memory_begin() {
    MEM_BEFORE_SWAP=$(swap_used_mb)
    MEM_BEFORE_PAGES=$(swap_pages)
}

memory_end() {
    local out=$1 label=$2 tmo=$3 after grew pages
    after=$(swap_used_mb)
    grew=$((after - MEM_BEFORE_SWAP))
    pages=$(( $(swap_pages) - MEM_BEFORE_PAGES ))
    memory_log "ran $label exit=$RUN_EXIT elapsed_s=$RUN_ELAPSED_S timeout_s=$tmo majflt=$RUN_MAJFLT maxrss_mb=$RUN_MAXRSS_MB swap_growth_mb=$grew swap_pages=$pages"
    if [ "$grew" -gt "$MAX_SWAP_GROWTH_MB" ] || [ "$pages" -gt "$MAX_SWAP_PAGES" ]; then
        echo "    contaminated: swap grew by ${grew} MB and paged ${pages} pages during $label; discarding the result" >&2
        rm -f "$out"
        return 1
    fi
    return 0
}

# bench OUT LABEL MEM_KB TMO_S CMD...: one guarded run; always logs the outcome
# to memory.log, and a failure or contamination removes the partial result so
# the table generators render "n/a"
bench() {
    local out=$1 label=$2 memkb=$3 tmo=$4 rc=0
    shift 4
    echo "... $label ..."
    require_memory "$label"
    memory_begin
    if ! run_checked "$out" "$label" "$memkb" "$tmo" "$@"; then
        rc=1
    fi
    if ! memory_end "$out" "$label" "$tmo"; then
        rc=1
    fi
    if [ "$rc" -ne 0 ]; then
        echo "    failed, timed out or contaminated; removing the partial result"
        rm -f "$out"
        return 1
    fi
    return 0
}

# A field of a matrix in data/matrices.json, e.g. matrix_field bwm2000 Symmetric
matrix_field() {
    python3 -c 'import json,sys
rows = json.load(open(sys.argv[3]))
print(next((m[sys.argv[2]] for m in rows if m["Name"] == sys.argv[1]), ""))' "$1" "$2" "$HERE/data/matrices.json"
}

# Solver flags per matrix: -p (positive definite) enables the Cholesky path
extra_args() {
    case "$1" in
        oilpan | inline_1 | PFlow_742 | Emilia_923 | Flan_1565 | pres-cylin-3d-tet10-fine) echo -p ;;
    esac
}

# run_solver GENIE MATS...: one solver over a list of matrices. UMFPACK and
# MUMPS run without an address-space cap, because their reservation is what is
# being measured; the Krylov methods are capped because the AMG setup on a
# million-dof matrix can ask for more than this machine has.
run_solver() {
    local genie=$1
    shift
    local outdir="$OUTPUT_DIR/$genie" memkb=0 tmo
    local mat
    case $genie in
        umfpack | mumps) tmo=$((NRUN * EXTERNAL_TIMEOUT + TIMEOUT_OVERHEAD)) ;;
        *) memkb=$ITERATIVE_MEMORY_KB tmo=$((NRUN * ITERATIVE_TIMEOUT + TIMEOUT_OVERHEAD)) ;;
    esac
    mkdir -p "$outdir"
    for mat in "$@"; do
        # CG requires a symmetric positive definite matrix: the solver says so
        # and the row is better left empty than reported as a failure
        if [ "$genie" = cg ] && [ "$(matrix_field "$mat" Symmetric)" != Yes ]; then
            echo "... $genie/$mat ... skipping (not symmetric)"
            memory_log "skip $genie/$mat reason=not_symmetric"
            rm -f "$outdir/$genie-$mat.json"
            continue
        fi
        bench "$outdir/$genie-$mat.json" "$genie/$mat" "$memkb" "$tmo" \
            "$COMMAND" -g "$genie" -r "$NRUN" $(extra_args "$mat") "$INPUT_DIR/$mat.mtx" || true
    done
}

# Runs the Direct solver on one matrix, with a probe pass to decide how many
# runs are worth doing
run_direct_matrix() {
    local mat=$1 extra flag="" outdir="$OUTPUT_DIR/direct" probe
    local elapsed nrun nnz
    extra=$(extra_args "$mat")
    if [ "$DIRECT_SUPERNODAL" = 1 ]; then
        outdir="$OUTPUT_DIR/direct-supernodal"
        flag=--direct-supernodal
    fi
    nnz=$(matrix_field "$mat" "Pattern Entries")
    if [ "$DIRECT_MAX_NNZ" -gt 0 ] && [ "${nnz:-0}" -gt "$DIRECT_MAX_NNZ" ]; then
        echo "... direct/$mat ... skipping (pattern entries $nnz > $DIRECT_MAX_NNZ)"
        memory_log "skip direct/$mat reason=max_nnz nnz=$nnz limit=$DIRECT_MAX_NNZ"
        rm -f "$outdir/direct-$mat.json"
        return
    fi
    mkdir -p "$outdir"
    probe="$outdir/.probe-$mat.json"

    if ! bench "$probe" "direct/$mat:probe" "$DIRECT_MEMORY_KB" "$DIRECT_PROBE_TIMEOUT" \
        "$COMMAND" -g direct -r 1 $flag $extra "$INPUT_DIR/$mat.mtx"; then
        echo "    probe failed or timed out; marking n/a"
        rm -f "$probe" "$outdir/direct-$mat.json"
        return
    fi
    elapsed=${RUN_ELAPSED_S%.*}

    nrun=1
    if [ "$elapsed" -lt 2 ]; then
        nrun=$DIRECT_MAX_RUNS
    elif [ "$elapsed" -lt 30 ]; then
        nrun=$(( DIRECT_MAX_RUNS > 3 ? 3 : DIRECT_MAX_RUNS ))
    fi
    if [ "$nrun" -le 1 ]; then
        mv "$probe" "$outdir/direct-$mat.json"
        echo "    probe took ${elapsed}s; keeping the single run"
        return
    fi

    echo "    probe took ${elapsed}s; running $nrun times"
    if bench "$outdir/direct-$mat.json" "direct/$mat" "$DIRECT_MEMORY_KB" $((elapsed * nrun * 3 + 600)) \
        "$COMMAND" -g direct -r "$nrun" $flag $extra "$INPUT_DIR/$mat.mtx"; then
        rm -f "$probe"
    else
        echo "    full run failed or timed out; keeping the probe result"
        mv "$probe" "$outdir/direct-$mat.json"
    fi
}

# STAGES selects the stages to run: all, or one of external, direct, iterative
want() { [ "$STAGES" = all ] || [ "$STAGES" = "$1" ]; }

# Verify the inputs before starting
for mat in "${REAL_MATS[@]}" "${COMPLEX_MATS[@]}"; do
    if [ ! -f "$INPUT_DIR/$mat.mtx" ]; then
        echo "error: missing $INPUT_DIR/$mat.mtx"
        exit 1
    fi
done

# External solvers first so the long Direct pass can be interrupted without
# losing their results
if want external; then
    run_solver umfpack "${REAL_MATS[@]}" "${COMPLEX_MATS[@]}"
    run_solver mumps "${REAL_MATS[@]}" "${COMPLEX_MATS[@]}"
fi

if want direct; then
    for mat in "${REAL_MATS[@]}" "${COMPLEX_MATS[@]}"; do
        run_direct_matrix "$mat"
    done
fi

# The iterative solvers are real-valued only: CG needs a symmetric matrix, and
# the preconditioner is built from the same real pattern
if want iterative; then
    run_solver cg "${REAL_MATS[@]}"
    run_solver bicgstab "${REAL_MATS[@]}"
    run_solver gmres "${REAL_MATS[@]}"
fi

echo "done: results in $OUTPUT_DIR"
