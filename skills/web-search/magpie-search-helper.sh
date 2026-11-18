#!/bin/bash
# magpie-search-helper.sh — Convenience wrapper for the Web Search skill
#
# Usage:
#   ./magpie-search-helper.sh <query> [csv|json] [output] [limit] [engine]
#
# Engines:
#   duckduckgo  Free, no API key required (default)
#   serper      Google Search API via Serper.dev (requires SERPER_API_KEY)
#
# Examples:
#   ./magpie-search-helper.sh "marketing agencies nyc" csv leads.csv
#   ./magpie-search-helper.sh "SEO consultants london" json results.json 20 serper
#   SERPER_API_KEY=... ./magpie-search-helper.sh "PPC agencies" csv ppc.csv 10 serper

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <query> [csv|json] [output] [limit] [engine]"
    echo ""
    echo "Engines: duckduckgo (default, free) | serper (requires SERPER_API_KEY)"
    echo ""
    echo "Examples:"
    echo "  $0 \"marketing agencies nyc\" csv leads.csv"
    echo "  $0 \"SEO consultants london\" json results.json 20 serper"
    exit 1
fi

QUERY="$1"
FORMAT="${2:-csv}"
OUTPUT="${3:-}"
LIMIT="${4:-10}"
ENGINE="${5:-duckduckgo}"

# Determine default output filename
if [ -z "$OUTPUT" ]; then
    OUTPUT="search.${FORMAT}"
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 "${SCRIPT_DIR}/magpie-search.py" "$QUERY" --format "$FORMAT" --out "$OUTPUT" --limit "$LIMIT" --engine "$ENGINE"
