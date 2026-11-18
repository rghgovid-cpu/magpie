---
name: Web Search
description: Search the web and return structured results (title, URL, snippet) that can be fed into the crawler skill for CRM extraction.
tags: search, web, google, duckduckgo, serper, query, results, find
---

# Web Search Skill

Use this skill to search the web and get structured results. Results include title, URL, and snippet — perfect for feeding into the Web Crawler skill for CRM lead extraction.

## Search Engine

Two engines are available:

| Engine | API key required? | Notes |
|--------|-------------------|-------|
| `duckduckgo` | No | DuckDuckGo HTML scraping. Free, but results can be limited/blocked. |
| `serper` | Yes | Serper.dev Google Search API. Higher-quality results. Requires a `SERPER_API_KEY`. |

Get a free Serper API key at https://serper.dev and set it as an environment variable:

```bash
export SERPER_API_KEY="your-serper-api-key"
```

Or store it in Magpie config:

```bash
magpie config serper-setup
# then export it for the skill to pick up:
export SERPER_API_KEY="$(magpie config serper-show --raw)"
```

## Output Fields

| Field | Description |
|-------|-------------|
| `query` | The search query used |
| `rank` | Result position (1, 2, 3...) |
| `title` | Page title from search result |
| `url` | Result URL |
| `snippet` | Search result description/snippet |
| `source` | Search engine used (`duckduckgo` or `serper`) |

## Commands

### Search the web (CSV output, DuckDuckGo)
```bash
magpie run "search for marketing agencies in New York --format csv --out marketing-agencies.csv"
```

### Search the web (JSON output, DuckDuckGo)
```bash
magpie run "search for SEO consultants London --format json --out seo-consultants.json"
```

### Search using Serper (requires SERPER_API_KEY)
```bash
magpie run "search for PPC agencies in Berlin --format csv --out ppc.csv --limit 10 --engine serper"
```

### Via the helper script
```bash
./magpie-search-helper.sh "marketing agencies nyc" csv leads.csv 10 duckduckgo
./magpie-search-helper.sh "SEO consultants london" json results.json 20 serper
```

## Example Workflow: Search → Crawl → CRM

```bash
# Step 1: Search for marketing agencies
magpie run "search for digital marketing agencies in Chicago --format csv --out search-results.csv"

# Step 2: Crawl the top results from the search
magpie run "crawl https://agency1.com https://agency2.com --format csv --out agency-leads.csv"

# Step 3: Import agency-leads.csv into your CRM (HubSpot, Salesforce, etc.)
```

## Example CSV Output

```csv
query,rank,title,url,snippet,source
digital marketing agencies chicago,1,"WebFX - Digital Marketing Agency","https://www.webfx.com/","Award-winning digital marketing agency in Chicago...",serper
digital marketing agencies chicago,2,"Straight North - Chicago SEO","https://www.straightnorth.com/","Chicago-based SEO and digital marketing firm...",serper
```

## Notes

- DuckDuckGo: no API key required. Results limited to 10 by default (use `--limit` to change).
- Serper: requires `SERPER_API_KEY`. Results limited to 10 by default.
- Snippets are truncated to 300 characters.
- Works well combined with the Web Crawler skill for lead generation.
