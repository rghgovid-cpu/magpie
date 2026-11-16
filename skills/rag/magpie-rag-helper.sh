#!/bin/bash
# magpie-rag-helper.sh — Convenience wrapper for the RAG skill
#
# Usage:
#   ./magpie-rag-helper.sh ingest
#   ./magpie-rag-helper.sh search "query text"
#   ./magpie-rag-helper.sh ingest-and-search "query text"
#   ./magpie-rag-helper.sh stats
#
# Examples:
#   ./magpie-rag-helper.sh ingest
#   ./magpie-rag-helper.sh search "enterprise pricing tiers"
#   ./magpie-rag-helper.sh search "brand voice guidelines"

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <ingest|search|stats|ingest-and-search> [query]"
    echo ""
    echo "Examples:"
    echo "  $0 ingest"
    echo "  $0 search \"enterprise pricing\""
    echo "  $0 ingest-and-search \"brand voice\""
    echo "  $0 stats"
    exit 1
fi

COMMAND="$1"
shift

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

case "$COMMAND" in
    ingest)
        python3 "${SCRIPT_DIR}/magpie-rag.py" ingest
        ;;
    search)
        if [ $# -lt 1 ]; then
            echo "Error: search requires a query argument" >&2
            exit 1
        fi
        python3 "${SCRIPT_DIR}/magpie-rag.py" search "$@"
        ;;
    stats)
        python3 "${SCRIPT_DIR}/magpie-rag.py" stats
        ;;
    ingest-and-search)
        if [ $# -lt 1 ]; then
            echo "Error: ingest-and-search requires a query argument" >&2
            exit 1
        fi
        python3 "${SCRIPT_DIR}/magpie-rag.py" ingest-and-search "$@"
        ;;
    *)
        echo "Unknown command: $COMMAND" >&2
        echo "Valid commands: ingest, search, stats, ingest-and-search" >&2
        exit 1
        ;;
esac
