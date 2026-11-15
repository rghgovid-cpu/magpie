#!/bin/bash
# magpie-print-helper.sh — Convenience wrapper for the Print skill
#
# Usage:
#   ./magpie-print-helper.sh <input.md> [pdf|word] [a4|letter] [output]
#
# Examples:
#   ./magpie-print-helper.sh launch.md pdf a4 launch.pdf
#   ./magpie-print-helper.sh launch.md word letter launch.docx

set -e

INPUT="${1:?Usage: $0 <input.md> [pdf|word] [a4|letter] [output]}"
FORMAT="${2:-pdf}"
SIZE="${3:-a4}"
OUTPUT="${4:-}"

if [ ! -f "$INPUT" ]; then
    echo "Error: File '$INPUT' not found" >&2
    exit 1
fi

ARGS=("print" "$INPUT" "--format" "$FORMAT" "--size" "$SIZE")
if [ -n "$OUTPUT" ]; then
    ARGS+=("--out" "$OUTPUT")
fi

magpie "${ARGS[@]}"
