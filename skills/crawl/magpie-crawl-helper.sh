#!/bin/bash
# magpie-crawl-helper.sh — Convenience wrapper for the Web Crawler skill
#
# Usage:
#   ./magpie-crawl-helper.sh <url> [csv|json] [output] [--limit N]
#
# Daily limit: 200 records per day by default (resets at midnight UTC).
# Use --limit to override for a single run.
#
# Examples:
#   ./magpie-crawl-helper.sh https://example.com csv leads.csv
#   ./magpie-crawl-helper.sh https://site1.com https://site2.com json leads.json --limit 50

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <url> [csv|json] [output] [--limit N]"
    echo ""
    echo "Daily limit: 200 records per day (resets at midnight UTC)."
    echo ""
    echo "Examples:"
    echo "  $0 https://example.com csv leads.csv"
    echo "  $0 https://site1.com https://site2.com json leads.json --limit 50"
    exit 1
fi

URLS=()
FORMAT="csv"
OUTPUT=""
EXTRA_ARGS=()

while [ $# -gt 0 ]; do
    arg="$1"
    case "$arg" in
        csv|json)
            FORMAT="$arg"
            shift
            ;;
        --limit)
            EXTRA_ARGS+=("--limit")
            shift
            if [ $# -gt 0 ]; then
                EXTRA_ARGS+=("$1")
                shift
            fi
            ;;
        --limit=*)
            EXTRA_ARGS+=("--limit" "${arg#--limit=}")
            shift
            ;;
        -*)
            EXTRA_ARGS+=("$arg")
            shift
            ;;
        *)
            if [[ "$arg" == http* ]]; then
                URLS+=("$arg")
            else
                OUTPUT="$arg"
            fi
            shift
            ;;
    esac
done

if [ ${#URLS[@]} -eq 0 ]; then
    echo "Error: No URLs provided" >&2
    exit 1
fi

if [ -z "$OUTPUT" ]; then
    OUTPUT="leads.${FORMAT}"
fi

python3 "$(dirname "$0")/magpie-crawl.py" "${URLS[@]}" --format "$FORMAT" --out "$OUTPUT" "${EXTRA_ARGS[@]}"
