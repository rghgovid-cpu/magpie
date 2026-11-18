#!/usr/bin/env python3
"""
campaign-research.py — Research businesses via Google Maps (Serper.dev)
for a Magpie marketing campaign.

Usage:
    python3 campaign-research.py --industry <industry> --location <location>
        [--limit <n>] [--out <file>]

Environment:
    SERPER_API_KEY   API key for Serper.dev (required).
                     Get one at https://serper.dev

Output:
    JSON array of leads with name, address, phone, website, rating, reviews, category.
"""

import sys
import os
import json
import argparse
from datetime import datetime, timezone

try:
    import urllib.request
    import urllib.error
    HAS_URLLIB = True
except ImportError:
    HAS_URLLIB = False


def search_serper_maps(industry, location, limit=20, api_key=None):
    """Search Google Maps via Serper.dev for businesses matching industry + location."""
    if not api_key:
        api_key = os.environ.get("SERPER_API_KEY", "")
    if not api_key:
        print(
            "Error: SERPER_API_KEY is not set. Export it or run 'magpie config serper-setup'.",
            file=sys.stderr,
        )
        return []

    query = f"{industry} in {location}"
    url = "https://maps.serper.dev/maps"
    body = {
        "q": query,
        "gl": "us",
        "hl": "en",
        "num": min(max(limit, 1), 20),
    }

    try:
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
    except Exception as e:
        print(f"Error searching Serper Maps: {e}", file=sys.stderr)
        return []

    leads = []
    places = payload.get("places", []) or payload.get("map", []) or []

    for item in places[:limit]:
        lead = {
            "name": item.get("title", "") or item.get("name", ""),
            "address": item.get("address", ""),
            "phone": item.get("phone", ""),
            "website": item.get("website", ""),
            "rating": item.get("rating"),
            "reviews": item.get("reviews"),
            "category": item.get("category", ""),
            "source": "serper-maps",
            "discovered_at": datetime.now(timezone.utc).isoformat(),
        }
        if lead["name"]:
            leads.append(lead)

    return leads


def main():
    parser = argparse.ArgumentParser(
        description="Magpie Campaign Research — find businesses via Google Maps (Serper)"
    )
    parser.add_argument(
        "--industry",
        required=True,
        help="Industry to research (e.g. dentist, plumber, marketing agency)",
    )
    parser.add_argument(
        "--location",
        required=True,
        help="Location to search (e.g. Chicago, London, New York)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Maximum number of leads to find (default: 20)",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Output JSON file (default: leads.json)",
    )
    args = parser.parse_args()

    leads = search_serper_maps(args.industry, args.location, limit=args.limit)

    print(f"Found {len(leads)} business(es) for {args.industry} in {args.location}",
          file=sys.stderr)

    # Determine output file
    if args.out:
        output_file = args.out
    else:
        output_file = "leads.json"

    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(leads, f, indent=2, ensure_ascii=False)

    print(f"Leads written to {output_file}", file=sys.stderr)

    # Print summary
    for i, lead in enumerate(leads, 1):
        print(f"{i}. {lead['name']}")
        if lead['address']:
            print(f"   {lead['address']}")
        if lead['phone']:
            print(f"   {lead['phone']}")
        if lead['website']:
            print(f"   {lead['website']}")
        print()


if __name__ == "__main__":
    main()
