#!/bin/bash
#
# Extract assembly code for algorithm variants (Rust + C)
# Generates both baseline and native-optimized ASM for comparison.
#
# Usage: ./scripts/extract_asm.sh <algorithm>
#
# Example:
#   ./scripts/extract_asm.sh dot_product

set -e

ALGO="${1:-dot_product}"

# Output directory - clean it first
ASM_DIR="asm_output"
rm -rf "$ASM_DIR"
mkdir -p "$ASM_DIR"

echo "═══════════════════════════════════════════════════════════════"
echo "  ASM Extraction: $ALGO"
echo "═══════════════════════════════════════════════════════════════"
echo ""

PROJECT_DIR="$(pwd)"

# Function to extract Rust ASM
extract_rust_asm() {
    local suffix="$1"
    local rustflags="$2"
    
    echo "→ Building Rust ($suffix)..."
    RUSTFLAGS="$rustflags --emit=asm" cargo build --release 2>&1 | grep -v "^warning:" || true
    
    MAIN_ASM=$(find target/release/deps -name "micro_optimize_algo-*.s" -type f 2>/dev/null | head -1)
    
    if [ -n "$MAIN_ASM" ]; then
        OUTPUT="$ASM_DIR/${ALGO}_rust_${suffix}.s"
        grep -A 100 "${ALGO}" "$MAIN_ASM" | head -500 > "$OUTPUT" 2>/dev/null || true
        echo "  ✓ $OUTPUT ($(wc -l < "$OUTPUT" | tr -d ' ') lines)"
    fi
}

# Function to extract C ASM  
extract_c_asm() {
    local suffix="$1"
    
    echo "→ Extracting C ($suffix)..."
    
    C_LIB=$(find target/release/build -name "libdot_product_c.a" -type f 2>/dev/null | head -1)
    
    if [ -n "$C_LIB" ]; then
        OUTPUT="$ASM_DIR/${ALGO}_c_${suffix}.s"
        echo "# C Assembly - $suffix" > "$OUTPUT"
        echo "# Source: $C_LIB" >> "$OUTPUT"
        
        TEMP_DIR=$(mktemp -d)
        (cd "$TEMP_DIR" && ar -x "$PROJECT_DIR/$C_LIB") 2>/dev/null || true
        
        for obj in "$TEMP_DIR"/*.o; do
            if [ -f "$obj" ]; then
                base=$(basename "${obj%.o}")
                echo "" >> "$OUTPUT"
                echo "# ═══════════════════════════════════════" >> "$OUTPUT"
                echo "# $base" >> "$OUTPUT"  
                echo "# ═══════════════════════════════════════" >> "$OUTPUT"
                objdump -d "$obj" >> "$OUTPUT" 2>/dev/null || true
            fi
        done
        
        rm -rf "$TEMP_DIR"
        echo "  ✓ $OUTPUT ($(wc -l < "$OUTPUT" | tr -d ' ') lines)"
    else
        echo "  ⚠ No C library found"
    fi
}

# Step 1: Baseline (no native optimizations)
echo "════════════════════════════════════════════════════════════════"
echo "  BASELINE (default target)"
echo "════════════════════════════════════════════════════════════════"
cargo clean -p micro-optimize-algo 2>/dev/null || true
extract_rust_asm "baseline" ""
extract_c_asm "baseline"

echo ""

# Step 2: Native optimizations
echo "════════════════════════════════════════════════════════════════"
echo "  NATIVE (target-cpu=native)"
echo "════════════════════════════════════════════════════════════════"
cargo clean -p micro-optimize-algo 2>/dev/null || true
extract_rust_asm "native" "-C target-cpu=native"
extract_c_asm "native"

echo ""

# List Rust functions
echo "════════════════════════════════════════════════════════════════"
echo "  Rust functions found:"
echo "════════════════════════════════════════════════════════════════"
MAIN_ASM=$(find target/release/deps -name "micro_optimize_algo-*.s" -type f 2>/dev/null | head -1)
if [ -n "$MAIN_ASM" ]; then
    grep "^_.*${ALGO}.*:$" "$MAIN_ASM" 2>/dev/null | sed 's/:$//' | while read -r func; do
        demangled=$(echo "$func" | c++filt 2>/dev/null || echo "$func")
        echo "  - $demangled"
    done
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "  Output files:"
echo "════════════════════════════════════════════════════════════════"
ls -la "$ASM_DIR"/*.s 2>/dev/null || echo "  (no files)"

echo ""
echo "Tips:"
echo "  - Compare baseline vs native: diff ${ASM_DIR}/${ALGO}_rust_baseline.s ${ASM_DIR}/${ALGO}_rust_native.s"
echo "  - Compare Rust vs C:          diff ${ASM_DIR}/${ALGO}_rust_native.s ${ASM_DIR}/${ALGO}_c_native.s"
