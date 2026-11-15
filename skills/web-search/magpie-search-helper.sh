#!/bin/bash
# magpie-search-helper.sh — Convenience wrapper for the Web Search skill
#
# Usage:
#   ./magpie-search-helper.sh <query> [csv|json] [output] [limit]
#
# Examples:
#   ./magpie-search-helper.sh "marketing agencies nyc" csv leads.csv
#   ./magpie-search-helper.sh "SEO consultants london" json results.json 20

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <query> [csv|json] [output] [limit]"
    echo ""
    echo "Examples:"
    echo "  $0 \"marketing agencies nyc\" csv leads.csv"
    echo "  $0 \"SEO consultants london\" json results.json 20"
    exit 1
fi

QUERY="$1"
FORMAT="${2:-csv}"
OUTPUT="${3:-}"
LIMIT="${4:-10}"

# Determine default output filename
if [ -z "$OUTPUT" ]; then
    OUTPUT="search.${FORMAT}"
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 "${SCRIPT_DIR}/magpie-search.py" "$QUERY" --format "$FORMAT" --out "$OUTPUT" --limit "$LIMIT"
