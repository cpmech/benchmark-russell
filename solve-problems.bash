#!/bin/bash

set -euo pipefail

# Number of runs
NRUN=10

# Constants
FEATURES="intel_mkl,local_sparse,cudss"
COMMAND="$HOME/rust_modules/release/solve_matrix_market"
INPUT_DIR="$HOME/Downloads/matrix-market"

# Detect the system
SYSTEM=$(grep -oP '^ID=\K.*' /etc/os-release | tr -d '"')
OUTPUT_DIR="results/$SYSTEM"

# Full matrix list
MATS="
    bwm2000 \
    rdb5000 \
    Goodwin_040 \
    fp \
    xenon1 \
    twotone \
    Raj1 \
    boyd2 \
    Goodwin_071 \
    darcy003 \
    rma10 \
    helm2d03 \
    stomach \
    oilpan \
    ASIC_680k \
    tmt_unsym \
    Goodwin_127 \
    pre2 \
    marine1 \
    torso1 \
    atmosmodd \
    atmosmodl \
    memchip \
    Freescale1 \
    rajat31 \
    Transport \
    inline_1 \
    PFlow_742 \
    Emilia_923 \
    dielFilterV2real \
    Flan_1565 \
    pres-cylin-3d-tet10-fine
"

# Matrix list for UMFPACK
# MATS="
#     bwm2000 \
#     rdb5000 \
#     Goodwin_040 \
#     fp \
#     xenon1 \
#     twotone \
#     Raj1 \
#     boyd2 \
#     Goodwin_071 \
#     darcy003 \
#     rma10 \
#     helm2d03 \
#     stomach \
#     oilpan \
#     ASIC_680k \
#     tmt_unsym \
#     Goodwin_127 \
#     marine1 \
#     torso1 \
#     memchip \
#     Freescale1 \
#     rajat31 \
# "

# Complex matrices
MATS="
    mhd1280b \
    mplate \
    RFdevice \
    vfem \
    fem_filter \
    Chevron4 \
    mono_500Hz \
    kim2 \
    fem_hifreq_circuit \
    dielFilterV3clx \
"

HERE=$(pwd)
cd ../russell/russell_sparse
cargo clean
cargo build --release --features "$FEATURES"
cd "$HERE"

mkdir -p "$OUTPUT_DIR"

run () {
    local genie=$1
    echo
    echo "=== $genie ==="
    mkdir -p "$OUTPUT_DIR/$genie"
    for mat in $MATS; do
        echo "... $mat ..."
        extra_args=""
        if [ "$mat" = "Goodwin_040" ]; then
            extra_args="--matching-gen MaxMinDiag"
        fi
        if [ "$mat" = "darcy003" ]; then
            extra_args="--matching-sym MaxMinDiagAlt"
        fi
        if [ "$mat" = "Goodwin_071" ]; then
            extra_args="--matching-gen MaxMinDiag"
        fi
        if [ "$mat" = "Goodwin_127" ]; then
            extra_args="--matching-gen MaxMinDiag"
        fi
        if [ "$mat" = "torso1" ]; then
            extra_args="--matching-gen MaxDiagCount"
        fi
        if [ "$mat" = "pres-cylin-3d-tet10-fine" ]; then
            extra_args="--hybrid-memory-factor 0.8 -p"
        fi
        # positive definite matrices
        if [ "$mat" = "oilpan" ]; then
            extra_args="-p"
        fi
        if [ "$mat" = "inline_1" ]; then
            extra_args="-p"
        fi
        if [ "$mat" = "PFlow_742" ]; then
            extra_args="-p"
        fi
        if [ "$mat" = "Emilia_923" ]; then
            extra_args="-p"
        fi
        if [ "$mat" = "Flan_1565" ]; then
            extra_args="-p"
        fi
        # complex matrices
        if [ "$mat" = "RFdevice" ]; then
            extra_args="--matching-gen MaxDiagSum"
        fi
        $COMMAND -g "$genie" -r "$NRUN" $extra_args "$INPUT_DIR/$mat.mtx" > "$OUTPUT_DIR/$genie/$genie-$mat.json"
    done
}

run cudss
run mumps
run umfpack
