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
#   ITERATIVE_MEMORY_KB  per-run memory cap, KiB (default: 8000000, ~8 GB)
#   ITERATIVE_TIMEOUT    per-run time cap, s    (default: 1200)

NRUN=${NRUN:-5}
DIRECT_MAX_RUNS=${DIRECT_MAX_RUNS:-5}
DIRECT_PROBE_TIMEOUT=${DIRECT_PROBE_TIMEOUT:-180}
DIRECT_MEMORY_KB=${DIRECT_MEMORY_KB:-20000000}
DIRECT_MAX_NNZ=${DIRECT_MAX_NNZ:-0}
DIRECT_SUPERNODAL=${DIRECT_SUPERNODAL:-0}
STAGES=${STAGES:-all}
ITERATIVE_MEMORY_KB=${ITERATIVE_MEMORY_KB:-8000000}
ITERATIVE_TIMEOUT=${ITERATIVE_TIMEOUT:-1200}

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
TARGET_DIR=${CARGO_TARGET_DIR:-$(cd "$RUSSELL_DIR/russell_sparse" && cargo metadata --format-version 1 --no-deps | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')}
COMMAND="$TARGET_DIR/release/solve_matrix_market"
if [ ! -x "$COMMAND" ]; then
    echo "error: cannot find $COMMAND"
    exit 1
fi

OUTPUT_DIR="$HERE/results/$SYSTEM"
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
# while the compute-bound factorization grows only two to three times. The
# numbers in this directory are therefore only meaningful on a machine that is
# not swapping, and the guard below enforces that instead of trusting the
# operator: it refuses to start a matrix when free memory is short, and it
# discards a result whose own run pushed pages into swap.
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
    if [ -r "$f" ]; then
        v=$(cat "$f")
        if [ "$v" = "max" ]; then echo 0; else echo $((v / 1024 / 1024)); fi
    else
        echo 0
    fi
}

memory_log() {
    echo "$(date -Is) $* avail_mb=$(mem_available_mb) swap_used_mb=$(swap_used_mb)" >> "$MEMORY_LOG"
}

# require_memory LABEL: abort the stage when the machine cannot host a solve
require_memory() {
    local avail
    avail=$(mem_available_mb)
    memory_log "check $1"
    if [ "$avail" -lt $((MIN_AVAILABLE_GB * 1024)) ]; then
        {
            echo "error: only ${avail} MB available, below MIN_AVAILABLE_GB=${MIN_AVAILABLE_GB} GB"
            echo "       MUMPS reserves up to 39 GB here; a solve that starts swapping is 50-100x slower"
            echo "       and its timing is worthless. Reclaim memory (reset the swap, stop other jobs)"
            echo "       or lower MIN_AVAILABLE_GB deliberately."
            echo "       See the comment above and results/*/memory.log."
        } >&2
        exit 1
    fi
    local cg
    cg=$(cgroup_memory_max_mb)
    memory_log "cgroup_max_mb=$cg"
    if [ "$cg" -gt 0 ] && [ "$cg" -lt "$MIN_CGROUP_MB" ]; then
        {
            echo "error: this process is capped by its cgroup to ${cg} MB (memory.max), below"
            echo "       MIN_CGROUP_MB=${MIN_CGROUP_MB}. A job scope of a few GiB turns a 25 GiB workspace"
            echo "       into permanent paging: measured 6.19 M major faults and 8.9 s where the same"
            echo "       binary takes 0.37 s uncapped. Run the suite outside the capped scope (no big"
            echo "       difference between host free memory and the result would show it otherwise)."
        } >&2
        exit 1
    fi
}

# run_checked OUT LABEL CMD...: run under a major-fault bound (rusage of the child)
run_checked() {
    local out=$1 label=$2
    shift 2
    python3 - "$out" "$label" "$MAX_MAJFLT" "$@" <<'PY'
import resource, subprocess, sys
out, label, limit, cmd = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4:]
before = resource.getrusage(resource.RUSAGE_CHILDREN)
with open(out, 'w') as f:
    rc = subprocess.call(cmd, stdout=f, stderr=subprocess.DEVNULL)
after = resource.getrusage(resource.RUSAGE_CHILDREN)
maj = after.ru_majflt - before.ru_majflt
print(f"    {label}: exit={rc} majflt={maj} maxrss_mb={after.ru_maxrss // 1024}", file=sys.stderr)
if maj > limit:
    print(f"    contaminated: {maj} major faults during {label}; discarding the result", file=sys.stderr)
    sys.exit(3)
sys.exit(rc)
PY
}

# memory_begin / memory_end OUT LABEL: discard a result whose run paged
memory_begin() {
    MEM_BEFORE_SWAP=$(swap_used_mb)
    MEM_BEFORE_PAGES=$(swap_pages)
}

memory_end() {
    local out=$1 label=$2 after grew pages
    after=$(swap_used_mb)
    grew=$((after - MEM_BEFORE_SWAP))
    pages=$(( $(swap_pages) - MEM_BEFORE_PAGES ))
    memory_log "ran $label swap_growth_mb=$grew swap_pages=$pages"
    if [ "$grew" -gt "$MAX_SWAP_GROWTH_MB" ] || [ "$pages" -gt "$MAX_SWAP_PAGES" ]; then
        echo "    contaminated: swap grew by ${grew} MB and paged ${pages} pages during $label; discarding the result" >&2
        rm -f "$out"
        return 1
    fi
    return 0
}

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
        require_memory "$genie/$mat"
        memory_begin
        if ! run_checked "$outdir/$genie-$mat.json" "$genie/$mat" \
                timeout 3600 "$COMMAND" -g "$genie" -r "$NRUN" $extra "$INPUT_DIR/$mat.mtx"; then
            echo "    failed, timed out or contaminated; removing the partial result"
            rm -f "$outdir/$genie-$mat.json"
        else
            memory_end "$outdir/$genie-$mat.json" "$genie/$mat" || true
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
    require_memory "direct/$mat"
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
            memory_begin
            if ! (
                ulimit -v "$DIRECT_MEMORY_KB"
                exec timeout $((elapsed * nrun * 3 + 600)) "$COMMAND" -g direct -r "$nrun" $direct_flag $extra "$INPUT_DIR/$mat.mtx"
            ) > "$outdir/direct-$mat.json" 2> /dev/null; then
                echo "    full run failed or timed out; keeping the probe result"
                mv "$probe" "$outdir/direct-$mat.json"
            else
                memory_end "$outdir/direct-$mat.json" "direct/$mat" || true
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

# Tells whether a matrix is symmetric, read from data/matrices.json
is_symmetric() {
    python3 -c 'import json,sys
data = json.load(open(sys.argv[2]))
print("yes" if next((m["Symmetric"] for m in data if m["Name"] == sys.argv[1]), "No") == "Yes" else "no")' "$1" "$HERE/data/matrices.json"
}

# Runs one Krylov method through russell_sparse's iterative solver
#
# Each run is capped in memory and in time: the AMG setup on a million-dof matrix can ask for more
# than this machine has, and a killed shell loses every result after it. CG is skipped on the
# matrices that are not symmetric, because it requires a symmetric positive definite one -- the
# solver says so and the row is better left empty than reported as a failure.
run_iterative() {
    local genie=$1
    local list=$2
    local outdir="$OUTPUT_DIR/$genie"
    mkdir -p "$outdir"
    for mat in $list; do
        if [ "$genie" = "cg" ] && [ "$(is_symmetric "$mat")" != "yes" ]; then
            echo "... $genie / $mat ... skipping (not symmetric)"
            rm -f "$outdir/$genie-$mat.json"
            continue
        fi
        echo "... $genie / $mat ..."
        require_memory "$genie/$mat"
        memory_begin
        if ! (
            ulimit -v "$ITERATIVE_MEMORY_KB"
            exec timeout "$ITERATIVE_TIMEOUT" "$COMMAND" -g "$genie" -r "$NRUN" "$INPUT_DIR/$mat.mtx"
        ) > "$outdir/$genie-$mat.json" 2> /dev/null; then
            echo "    failed, killed or timed out; removing the partial result"
            rm -f "$outdir/$genie-$mat.json"
        else
            memory_end "$outdir/$genie-$mat.json" "$genie/$mat" || true
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
