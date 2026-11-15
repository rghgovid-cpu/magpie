#!/bin/bash
# magpie-crawl-helper.sh — Convenience wrapper for the Web Crawler skill
#
# Usage:
#   ./magpie-crawl-helper.sh <url> [csv|json] [output]
#
# Examples:
#   ./magpie-crawl-helper.sh https://example.com csv leads.csv
#   ./magpie-crawl-helper.sh https://site1.com https://site2.com json leads.json

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <url> [csv|json] [output]"
    echo ""
    echo "Examples:"
    echo "  $0 https://example.com csv leads.csv"
    echo "  $0 https://site1.com https://site2.com json leads.json"
    exit 1
fi

URLS=("$@")
# Last arg might be output file if it doesn't look like a URL
LAST="${URLS[-1]}"
FORMAT="csv"
OUTPUT=""

# Parse args: find format and output
POSITIONAL=()
for arg in "$@"; do
    case "$arg" in
        csv|json) FORMAT="$arg" ;;
        *) POSITIONAL+=("$arg") ;;
    esac
done

# Determine output
if [ ${#POSITIONAL[@]} -gt 0 ]; then
    # Check if last positional arg is a URL or output file
    LAST_POS="${POSITIONAL[-1]}"
    if [[ "$LAST_POS" == http* ]]; then
        # All are URLs, use default output
        URLS=("${POSITIONAL[@]}")
        OUTPUT="leads.${FORMAT}"
    else
        # Last is output file
        OUTPUT="$LAST_POS"
        URLS=("${POSITIONAL[@]:0:${#POSITIONAL[@]}-1}")
    fi
fi

if [ ${#URLS[@]} -eq 0 ]; then
    echo "Error: No URLs provided" >&2
    exit 1
fi

python3 "$(dirname "$0")/magpie-crawl.py" "${URLS[@]}" --format "$FORMAT" --out "$OUTPUT"
