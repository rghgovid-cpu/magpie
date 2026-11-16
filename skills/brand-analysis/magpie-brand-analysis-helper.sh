#!/bin/bash
# magpie-brand-analysis-helper.sh — Convenience wrapper for the Brand Analysis skill
#
# Usage:
#   ./magpie-brand-analysis-helper.sh <url> [css|json|md] [output] [model]
#
# Examples:
#   ./magpie-brand-analysis-helper.sh https://example.com css brand.css
#   ./magpie-brand-analysis-helper.sh https://site.com json brand.json
#   ./magpie-brand-analysis-helper.sh https://site.com md brand.md
#
# Requirements:
#   pip install playwright
#   playwright install chromium
#   MAGPIE_API_KEY or OPENAI_API_KEY environment variable
#   Vision-capable model (Nemotron-3.5-Super-VL-Free, GPT-4V, Claude 3, etc.)

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <url> [css|json|md] [output] [model]"
    echo ""
    echo "Examples:"
    echo "  $0 https://example.com css brand.css"
    echo "  $0 https://site.com json brand.json"
    echo "  $0 https://site.com md brand.md"
    echo ""
    echo "Environment variables:"
    echo "  MAGPIE_API_KEY or OPENAI_API_KEY  (required)"
    echo "  MAGPIE_BASE_URL                   (default: https://api.openai.com/v1)"
    echo "  MAGPIE_MODEL                      (default: nemotron-3.5-super-vl-free)"
    exit 1
fi

URL="$1"
FORMAT="${2:-css}"
OUTPUT="${3:-}"
MODEL="${4:-}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

ARGS=("$URL" --format "$FORMAT")
if [ -n "$OUTPUT" ]; then
    ARGS+=("--out" "$OUTPUT")
fi
if [ -n "$MODEL" ]; then
    ARGS+=("--model" "$MODEL")
fi

python3 "${SCRIPT_DIR}/magpie-brand-analysis.py" "${ARGS[@]}"
