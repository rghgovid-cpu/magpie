#!/usr/bin/env python3
"""
magpie-search.py — Web search for Magpie skill system.
Searches DuckDuckGo HTML (free, no API key) or Serper.dev (Google Search API)
and returns structured results (CSV/JSON).

Usage:
    python3 magpie-search.py <query> [--format csv|json] [--out <file>] [--limit <n>] [--engine duckduckgo|serper]

Environment:
    SERPER_API_KEY   API key for Serper.dev (required when --engine serper).
                     Get one at https://serper.dev
"""

import sys
import re
import os
import csv
import json
import argparse
import urllib.request
import urllib.error
from datetime import datetime, timezone
from urllib.parse import parse_qs, quote_plus, urlsplit
from html import unescape

try:
    import requests
    from bs4 import BeautifulSoup
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False


def search_duckduckgo(query, limit=10):
    """Search DuckDuckGo HTML and return structured results."""
    if HAS_REQUESTS:
        try:
            url = "https://html.duckduckgo.com/html/"
            params = {"q": query}
            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                              "AppleWebKit/537.36 (KHTML, like Gecko) "
                              "Chrome/120.0.0.0 Safari/537.36"
            }
            resp = requests.get(url, params=params, headers=headers, timeout=20)
            resp.raise_for_status()
            html = resp.text
        except Exception as e:
            print(f"Error searching DuckDuckGo: {e}", file=sys.stderr)
            return []
    else:
        # Fallback using urllib (standard library)
        try:
            import urllib.request
            url = "https://html.duckduckgo.com/html/?q=" + quote_plus(query)
            req = urllib.request.Request(
                url,
                headers={
                    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                                  "AppleWebKit/537.36 (KHTML, like Gecko) "
                                  "Chrome/120.0.0.0 Safari/537.36"

                }
            )
            with urllib.request.urlopen(req, timeout=20) as resp:
                html = resp.read().decode("utf-8", errors="replace")
        except Exception as e:
            print(f"Error searching DuckDuckGo: {e}", file=sys.stderr)
            return []

    soup = BeautifulSoup(html, "html.parser")
    results = []

    # DuckDuckGo HTML result selectors
    result_blocks = soup.select(".result") or soup.select(".web-result")

    for block in result_blocks[:limit]:
        try:
            # Title and URL
            title_el = block.select_one(".result__a") or block.select_one("a.result__a")
            if not title_el:
                # Try alternative selector
                title_el = block.find("a", class_=re.compile("result"))

            if not title_el:
                continue

            title = clean_text(title_el.get_text())
            url = title_el.get("href", "")

            # DuckDuckGo uses redirect URLs; extract the actual URL from uddg param
            if url and "uddg=" in url:
                parsed = urlsplit(url)
                qs = parse_qs(parsed.query)
                if "uddg" in qs:
                    url = qs["uddg"][0]

            # Snippet
            snippet_el = block.select_one(".result__snippet") or block.select_one(".snippet")
            snippet = clean_text(snippet_el.get_text()) if snippet_el else ""

            if not title or not url:
                continue

            results.append({
                "title": title,
                "url": url,
                "snippet": snippet[:300],  # truncate snippet
            })
        except Exception as e:
            print(f"Error parsing result: {e}", file=sys.stderr)
            continue

    return results


def search_serper(query, limit=10, api_key=None):
    """Search using Serper.dev Google Search API and return structured results."""
    if not api_key:
        api_key = os.environ.get("SERPER_API_KEY", "")
    if not api_key:
        print(
            "Error: SERPER_API_KEY is not set. Export it (e.g. export SERPER_API_KEY=...) "
            "or run 'magpie config serper-setup', then retry. Or use --engine duckduckgo.",
            file=sys.stderr,
        )
        return []

    try:
        url = "https://google.serper.dev/search"
        body = {
            "q": query,
            "num": min(max(limit, 1), 10),
        }
        data = json.dumps(body).encode("utf-8")
        req = urllib.request.Request(
            url,
            data=data,
            headers={
                "X-API-KEY": api_key,
                "Content-Type": "application/json",
                "User-Agent": "Magpie/1.0",
            },
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            payload = json.loads(resp.read().decode("utf-8", errors="replace"))
    except (urllib.error.URLError, urllib.error.HTTPError, json.JSONDecodeError, OSError) as e:
        print(f"Error searching Serper: {e}", file=sys.stderr)
        return []

    results = []
    for item in payload.get("organic", [])[:limit]:
        results.append({
            "title": item.get("title", ""),
            "url": item.get("link", ""),
            "snippet": item.get("snippet", ""),
        })

    return results


def clean_text(text):
    """Clean and normalize text."""
    if not text:
        return ""
    text = unescape(text)
    text = re.sub(r"\s+", " ", text)
    return text.strip()


def write_csv(records, query, output_file, source):
    """Write search results to CSV file."""
    fieldnames = ["query", "rank", "title", "url", "snippet", "source"]
    with open(output_file, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for i, record in enumerate(records, 1):
            writer.writerow({
                "query": query,
                "rank": i,
                "title": record.get("title", ""),
                "url": record.get("url", ""),
                "snippet": record.get("snippet", ""),
                "source": source,
            })


def write_json(records, query, output_file, source):
    """Write search results to JSON file."""
    output = {
        "query": query,
        "source": source,
        "search_date": datetime.now(timezone.utc).isoformat(),
        "total": len(records),
        "results": [
            {
                "rank": i,
                "title": r.get("title", ""),
                "url": r.get("url", ""),
                "snippet": r.get("snippet", ""),
            }
            for i, r in enumerate(records, 1)
        ],
    }
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)


def main():
    parser = argparse.ArgumentParser(
        description="Magpie Web Search — search the web and export results"
    )
    parser.add_argument("query", help="Search query")
    parser.add_argument(
        "--format",
        choices=["csv", "json"],
        default="csv",
        help="Output format (default: csv)",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Output file (default: search.csv or search.json)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=10,
        help="Maximum number of results (default: 10)",
    )
    parser.add_argument(
        "--engine",
        choices=["duckduckgo", "serper"],
        default="duckduckgo",
        help="Search engine to use (default: duckduckgo). 'serper' requires SERPER_API_KEY.",
    )
    args = parser.parse_args()

    # Perform search
    if args.engine == "serper":
        results = search_serper(args.query, limit=args.limit)
        source = "serper"
    else:
        results = search_duckduckgo(args.query, limit=args.limit)
        source = "duckduckgo"

    print(f"Found {len(results)} result(s) for: {args.query} (engine: {source})",
          file=sys.stderr)

    # Determine output file
    if args.out:
        output_file = args.out
    else:
        output_file = "search.csv" if args.format == "csv" else "search.json"

    # Write output
    if args.format == "csv":
        write_csv(results, args.query, output_file, source)
    else:
        write_json(results, args.query, output_file, source)

    print(f"Results written to {output_file}", file=sys.stderr)

    # Print results to stdout for immediate use
    for i, r in enumerate(results, 1):
        print(f"{i}. {r['title']}")
        print(f"   {r['url']}")
        if r['snippet']:
            print(f"   {r['snippet'][:200]}")
        print()


if __name__ == "__main__":
    main()
